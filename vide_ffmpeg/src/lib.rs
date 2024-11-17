use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use ac_ffmpeg::{
    codec::{
        video::{self, PixelFormat, VideoEncoder, VideoFrameMut},
        Encoder,
    },
    format::{
        io::IO,
        muxer::{Muxer, OutputFormat},
    },
    time::{TimeBase, Timestamp},
};
use vide_common::{config::RenderConfiguration, prelude::TimeCode, render::Wgpu, FrameInfo};
use vide_render::{interface::OutputHandler, texture_factory::FactoryTexture};

fn open_output(
    path: &str,
    elementary_streams: &[ac_ffmpeg::codec::CodecParameters],
) -> Result<Muxer<File>, ac_ffmpeg::Error> {
    let output_format = OutputFormat::guess_from_file_name(path).ok_or_else(|| {
        ac_ffmpeg::Error::new(format!("unable to guess output format for file: {}", path))
    })?;

    let output = File::create(path).map_err(|err| {
        ac_ffmpeg::Error::new(format!("unable to create output file {}: {}", path, err))
    })?;

    let io = IO::from_seekable_write_stream(output);

    let mut muxer_builder = Muxer::builder();

    for codec_parameters in elementary_streams {
        muxer_builder.add_stream(codec_parameters)?;
    }

    muxer_builder.build(io, output_format)
}

struct ConfiguredProperties {
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    unpadded_bytes_per_row: u32,
    buffer: wgpu::Buffer,
    pixel_format: PixelFormat,
    encoder: VideoEncoder,
    muxer: Muxer<File>,
}

pub struct MediaExporter {
    path: PathBuf,
    time_base: TimeBase,
    configured: Option<ConfiguredProperties>,
}

impl MediaExporter {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().into(),
            time_base: TimeBase::new(1, TimeCode::time_base() as _),
            configured: None,
        }
    }
}

impl OutputHandler for MediaExporter {
    fn configure(&mut self, wgpu: &Wgpu, config: &RenderConfiguration) -> wgpu::TextureFormat {
        let width = config.resolution.0 as u32;
        let height = config.resolution.1 as u32;

        let pixel_size = size_of::<[u8; 4]>() as u32;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unpadded_bytes_per_row = pixel_size * width;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;

        let buffer_size = (padded_bytes_per_row * height) as wgpu::BufferAddress;
        let buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Media Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let pixel_format = video::frame::get_pixel_format("rgb24");

        let encoder = VideoEncoder::builder("libx264rgb")
            .unwrap()
            .pixel_format(pixel_format)
            .time_base(self.time_base)
            .width(width as _)
            .height(height as _)
            .build()
            .unwrap();

        let codec_parameters = encoder.codec_parameters().into();
        let muxer = open_output(self.path.to_str().unwrap(), &[codec_parameters]).unwrap();

        self.configured = Some(ConfiguredProperties {
            width,
            height,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
            buffer,
            pixel_format,
            encoder,
            muxer,
        });

        wgpu::TextureFormat::Rgba8UnormSrgb
    }

    fn publish_frame(
        &mut self,
        wgpu: &Wgpu,
        mut command_encoder: wgpu::CommandEncoder,
        texture: &FactoryTexture,
        _frame: i64,
        frame_info: FrameInfo,
    ) {
        let ConfiguredProperties {
            width,
            height,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
            ref buffer,
            pixel_format,
            ref mut encoder,
            ref mut muxer,
        } = self.configured.as_mut().expect("Not configured yet");

        command_encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: texture.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(*padded_bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: *width,
                height: *height,
                depth_or_array_layers: 1,
            },
        );

        wgpu.queue.submit(std::iter::once(command_encoder.finish()));

        log::trace!("Copying frame");

        let (sender, receiver) = channel();

        let buffer_slice = buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap()
        });

        wgpu.device.poll(wgpu::Maintain::Wait);

        match receiver.recv().unwrap() {
            Ok(()) => {
                let padded_data = buffer_slice.get_mapped_range();

                let data = padded_data
                    .chunks(*padded_bytes_per_row as _)
                    .flat_map(|chunk| &chunk[..*unpadded_bytes_per_row as _])
                    .copied()
                    .collect::<Vec<_>>();

                drop(padded_data);
                buffer.unmap();

                let mapped_data = data
                    .chunks(4)
                    .flat_map(|c| &c[..3])
                    .copied()
                    .collect::<Vec<_>>();

                log::info!("Encoding frame");

                let mut new_frame = VideoFrameMut::black(*pixel_format, *width as _, *height as _);

                new_frame.planes_mut()[0]
                    .data_mut()
                    .write_all(&mapped_data)
                    .unwrap();

                let timestamp = Timestamp::new(frame_info.time_code.value(), self.time_base);
                encoder
                    .push(new_frame.with_pts(timestamp).freeze())
                    .unwrap();

                while let Some(packet) = encoder.take().unwrap() {
                    muxer.push(packet.with_stream_index(0)).unwrap();
                }
            }
            other => {
                log::error!("{:?}", other);
            }
        }
    }

    fn finish(&mut self, _wgpu: &Wgpu) {
        let ConfiguredProperties {
            ref mut encoder,
            ref mut muxer,
            ..
        } = self.configured.as_mut().expect("Not configured yet");

        encoder.flush().unwrap();

        while let Some(packet) = encoder.take().unwrap() {
            muxer.push(packet.with_stream_index(0)).unwrap();
        }

        muxer.flush().unwrap();
    }
}
