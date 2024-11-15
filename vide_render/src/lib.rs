use euler::Mat4;
use interface::OutputHandler;
use pollster::FutureExt;
use vide_common::{
    config::RenderConfiguration, prelude::TimeCode, render::Wgpu,
    time_code::UnboundedTimecodeRange, types::TimeUnit, FrameInfo,
};
use vide_project::{clip::Clip, Project};

pub mod interface;

pub async fn init_wgpu() -> Wgpu {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .expect("Unable to find a compatible adapter to render with");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Render Device"),
                memory_hints: wgpu::MemoryHints::Performance,
                required_features: wgpu::Features::all_native_mask(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Unable to find a compatible device to render with");

    Wgpu {
        instance,
        adapter,
        device,
        queue,
    }
}

fn render_clip(
    clip: &mut Clip,
    wgpu: &Wgpu,
    frame_info: FrameInfo,
    parent_range: UnboundedTimecodeRange,
    parent_transform: Mat4,
    encoder: &mut wgpu::CommandEncoder,
) {
    let absolute_range = clip.range().make_absolute(parent_range);

    if !absolute_range.contains(frame_info.time_code) {
        return;
    }

    let absolute_transform = parent_transform * clip.transform().matrix();

    for child in clip.children_mut() {
        render_clip(
            child,
            wgpu,
            frame_info,
            absolute_range,
            absolute_transform,
            encoder,
        );
    }

    let local_frame_info = frame_info.make_local(absolute_range);

    if let Some(video) = clip.video_mut() {
        video.set_transform(absolute_transform);
        video.update(wgpu, &frame_info, &local_frame_info);
        // video.render(wgpu, &frame_info, &local_frame_info, encoder, destination)
    }
}

pub fn render(project: &mut Project, config: RenderConfiguration, mut output: impl OutputHandler) {
    let wgpu = init_wgpu().block_on();
    let output_format = output.configure(&wgpu, &config);

    let output_texture = wgpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("OutputHandler read texture"),
        dimension: wgpu::TextureDimension::D2,
        format: output_format,
        mip_level_count: 1,
        sample_count: 1,
        size: wgpu::Extent3d {
            width: config.resolution.0 as u32,
            height: config.resolution.1 as u32,
            depth_or_array_layers: 1,
        },
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let project_range =
        UnboundedTimecodeRange::new(Some(TimeCode::new(0)), Some(project.duration()));
    let frames = project.frame_count(config.frames_per_second);

    for frame in 0..frames {
        let mut encoder = wgpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let time_code = TimeUnit::Seconds(frame as f64 / config.frames_per_second).into();
        let progress = frame as f64 / frames as f64;

        for clip in project.clips_mut() {
            render_clip(
                clip,
                &wgpu,
                FrameInfo {
                    time_code,
                    progress,
                    resolution: config.resolution,
                },
                project_range,
                Mat4::identity(),
                &mut encoder,
            );
        }
    }
}
