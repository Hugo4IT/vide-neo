use blend::BlendModes;
use euler::Mat4;
use interface::OutputHandler;
use pollster::FutureExt;
use texture_factory::{FactoryTexture, TextureFactory};
use vide_common::{
    config::RenderConfiguration, prelude::TimeCode, render::Wgpu, standards::FRAGMENT_COLOR_TARGET,
    time_code::UnboundedTimecodeRange, types::TimeUnit, FrameInfo,
};
use vide_project::{clip::Clip, Project};

pub mod blend;
pub mod interface;
pub mod texture_factory;

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
                required_features: wgpu::Features::all_native_mask()
                    | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
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

fn init_clip(clip: &mut Clip, wgpu: &Wgpu, config: &RenderConfiguration) {
    if let Some(video) = clip.video_mut() {
        video.init(wgpu, config);
    }

    for child in clip.children_mut() {
        init_clip(child, wgpu, config);
    }
}

fn render_clip(
    clip: &mut Clip,
    wgpu: &Wgpu,
    frame_info: FrameInfo,
    parent_range: UnboundedTimecodeRange,
    parent_transform: Mat4,
    encoder: &mut wgpu::CommandEncoder,
    texture_factory: &mut TextureFactory,
    blend_modes: &BlendModes,
) -> Option<FactoryTexture> {
    let absolute_range = clip.range().make_absolute(parent_range);

    if !absolute_range.contains(frame_info.time_code) {
        None?
    }

    let absolute_transform = parent_transform * clip.transform().matrix();

    let mut canvas_texture = texture_factory.borrow_texture(wgpu);
    let mut blended_texture = texture_factory.borrow_texture(wgpu);

    for child in clip.children_mut() {
        let output = render_clip(
            child,
            wgpu,
            frame_info,
            absolute_range,
            absolute_transform,
            encoder,
            texture_factory,
            blend_modes,
        );

        if let Some(output) = output {
            blend_modes.normal.blend(
                wgpu,
                encoder,
                output.view(),
                canvas_texture.view(),
                blended_texture.view(),
            );

            // Swap to reuse the textures
            core::mem::swap(&mut canvas_texture, &mut blended_texture);

            texture_factory.return_texture(output);
        }
    }

    let local_frame_info = frame_info.make_local(absolute_range);

    if let Some(video) = clip.video_mut() {
        let output_texture = texture_factory.borrow_texture(wgpu);

        video.set_transform(absolute_transform);
        video.update(wgpu, &frame_info, &local_frame_info);
        video.render(
            wgpu,
            &frame_info,
            &local_frame_info,
            encoder,
            output_texture.view(),
        );

        blend_modes.normal.blend(
            wgpu,
            encoder,
            output_texture.view(),
            canvas_texture.view(),
            blended_texture.view(),
        );

        texture_factory.return_texture(output_texture);
    }

    texture_factory.return_texture(canvas_texture);

    Some(blended_texture)
}

pub fn render(project: &mut Project, config: RenderConfiguration, mut output: impl OutputHandler) {
    let wgpu = init_wgpu().block_on();

    for clip in project.clips_mut() {
        init_clip(clip, &wgpu, &config);
    }

    let output_format = output.configure(&wgpu, &config);

    let mut handler_texture_factory = TextureFactory::new(
        wgpu::TextureDescriptor {
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
        },
        wgpu::TextureViewDescriptor::default(),
    );

    let handler_canvas_texture = handler_texture_factory.borrow_texture(&wgpu);
    let handler_blended_texture = handler_texture_factory.borrow_texture(&wgpu);

    let project_range =
        UnboundedTimecodeRange::new(Some(TimeCode::new(0)), Some(project.duration()));
    let frames = project.frame_count(config.frames_per_second);

    let mut texture_factory = TextureFactory::new(
        wgpu::TextureDescriptor {
            label: Some("OutputHandler read texture"),
            dimension: wgpu::TextureDimension::D2,
            format: FRAGMENT_COLOR_TARGET,
            mip_level_count: 1,
            sample_count: 1,
            size: wgpu::Extent3d {
                width: config.resolution.0 as u32,
                height: config.resolution.1 as u32,
                depth_or_array_layers: 1,
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::TextureViewDescriptor::default(),
    );

    let blend_modes = BlendModes::load(&wgpu, FRAGMENT_COLOR_TARGET);
    let blend_modes_root = BlendModes::load(&wgpu, output_format);

    for frame in 0..frames {
        let mut encoder = wgpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let time_code = TimeUnit::Seconds(frame as f64 / config.frames_per_second).into();
        let progress = frame as f64 / frames as f64;

        let mut canvas_texture = texture_factory.borrow_texture(&wgpu);
        let mut blended_texture = texture_factory.borrow_texture(&wgpu);

        for clip in project.clips_mut() {
            let output = render_clip(
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
                &mut texture_factory,
                &blend_modes,
            );

            if let Some(output) = output {
                blend_modes.normal.blend(
                    &wgpu,
                    &mut encoder,
                    output.view(),
                    canvas_texture.view(),
                    blended_texture.view(),
                );

                // Swap to reuse textures
                core::mem::swap(&mut canvas_texture, &mut blended_texture);

                texture_factory.return_texture(output);
            }
        }

        blend_modes_root.normal.blend(
            &wgpu,
            &mut encoder,
            blended_texture.view(),
            handler_canvas_texture.view(),
            handler_blended_texture.view(),
        );

        texture_factory.return_texture(canvas_texture);
        texture_factory.return_texture(blended_texture);

        output.publish_frame(&wgpu, handler_blended_texture.texture());
    }

    handler_texture_factory.return_texture(handler_canvas_texture);
    handler_texture_factory.return_texture(handler_blended_texture);
}
