use std::{
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use vide_common::{config::RenderConfiguration, render::Wgpu, FrameInfo};
use vide_render::{interface::OutputHandler, texture_factory::FactoryTexture};

struct ConfiguredProperties {
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    unpadded_bytes_per_row: u32,
    buffer: wgpu::Buffer,
}

pub struct MediaExporter {
    path: PathBuf,
    configured: Option<ConfiguredProperties>,
}

impl MediaExporter {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().into(),
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
            label: Some("GIF Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        self.configured = Some(ConfiguredProperties {
            width,
            height,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
            buffer,
        });

        wgpu::TextureFormat::Rgba8UnormSrgb
    }

    fn publish_frame(
        &mut self,
        wgpu: &Wgpu,
        mut encoder: wgpu::CommandEncoder,
        texture: &FactoryTexture,
        frame: i64,
        frame_info: FrameInfo,
    ) {
        let ConfiguredProperties {
            width,
            height,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
            ref buffer,
        } = self.configured.as_mut().expect("Not configured yet");

        encoder.copy_texture_to_buffer(
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

        wgpu.queue.submit(std::iter::once(encoder.finish()));

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

                log::info!("Encoding frame");

                // image::save_buffer(path, &data, *width, *height, image::ColorType::Rgba8).unwrap();
            }
            other => {
                log::error!("{:?}", other);
            }
        }
    }
}
