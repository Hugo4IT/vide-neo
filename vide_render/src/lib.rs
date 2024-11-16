use blend::BlendModes;
use euler::{mat4, Mat4};
use interface::OutputHandler;
use pollster::FutureExt;
use texture_factory::{FactoryTexture, TextureFactory};
use vide_common::{
    config::RenderConfiguration,
    prelude::TimeCode,
    render::{GlobalUniform, Wgpu},
    standards::FRAGMENT_COLOR_TARGET,
    time_code::UnboundedTimecodeRange,
    types::TimeUnit,
    FrameInfo,
};
use vide_project::{clip::Clip, Project};
use wgpu::util::DeviceExt;

pub mod blend;
pub mod export;
pub mod interface;
pub mod texture_factory;

pub async fn init_wgpu() -> Wgpu {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    log::info!("Requesting adapter");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .expect("Unable to find a compatible adapter to render with");

    log::info!("Requesting device");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Render Device"),
                memory_hints: wgpu::MemoryHints::Performance,
                required_features: wgpu::Features::empty()
                    | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Unable to find a compatible device to render with");

    log::info!("Setting up global bind group");

    let global_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Global Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let temporary_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Temporary Global Bind Group Layout"),
            entries: &[],
        });

    let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Temporary Global Bind Group"),
        layout: &temporary_bind_group_layout,
        entries: &[],
    });

    Wgpu {
        instance,
        adapter,
        device,
        queue,
        global_bind_group_layout,
        global_bind_group,
    }
}

fn init_clip(clip: &mut Clip, wgpu: &Wgpu, config: &RenderConfiguration, counter: &mut u32) {
    *counter += 1;

    log::trace!("Initializing clip at {}", clip.range());

    if let Some(video) = clip.video_mut() {
        video.init(wgpu, config);
    }

    for child in clip.children_mut() {
        init_clip(child, wgpu, config, counter);
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
            // Swap to reuse the textures
            core::mem::swap(&mut canvas_texture, &mut blended_texture);

            blend_modes.normal.blend(
                wgpu,
                encoder,
                output.view(),
                canvas_texture.view(),
                blended_texture.view(),
            );

            texture_factory.return_texture(output);
        }
    }

    let local_frame_info = frame_info.make_local(absolute_range);

    if let Some(video) = clip.video_mut() {
        // Swap to reuse the textures
        core::mem::swap(&mut canvas_texture, &mut blended_texture);

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

fn generate_ortho_matrix(config: &RenderConfiguration) -> Mat4 {
    let width = config.resolution.0 as f32;
    let height = config.resolution.1 as f32;

    let pixel_width = 2.0 / width;
    let pixel_height = 2.0 / height;
    let pixel_depth = -1.0 / 10.0;

    #[rustfmt::skip]
    let matrix = mat4!(
         pixel_width,  0.0,                 0.0,         0.0,
         0.0,                pixel_height,  0.0,         0.0,
         0.0,                0.0,           pixel_depth, 0.0,
         -1.0,                -1.0,           0.0,         1.0,
    );

    matrix
}

pub fn render(mut project: Project, config: RenderConfiguration, mut output: impl OutputHandler) {
    let _ = env_logger::try_init();

    log::info!("Initializing wgpu");

    let mut wgpu = init_wgpu().block_on();

    let global_uniform_buffer = wgpu
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GlobalUniform {
                ortho_matrix: generate_ortho_matrix(&config).into(),
            }]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

    wgpu.global_bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Global Bind Group"),
        layout: &wgpu.global_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: global_uniform_buffer.as_entire_binding(),
        }],
    });

    {
        log::info!("Initializing clips");

        let mut counter = 0u32;

        for clip in project.clips_mut() {
            init_clip(clip, &wgpu, &config, &mut counter);
        }

        log::info!("Initialized {counter} clips");
    }

    log::info!("Configuring output handler");

    let output_format = output.configure(&wgpu, &config);

    log::info!("Initializing texture factories and output textures");

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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        wgpu::TextureViewDescriptor::default(),
    );

    let handler_canvas_texture = handler_texture_factory.borrow_texture(&wgpu);
    let handler_blended_texture = handler_texture_factory.borrow_texture(&wgpu);

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

    let project_range =
        UnboundedTimecodeRange::new(Some(TimeCode::new(0)), Some(project.duration()));
    let frames = project.frame_count(config.frames_per_second);

    log::info!("Starting render ({frames} frames)");

    for frame in 0..frames {
        log::trace!("Rendering frame {frame}");

        let mut encoder = wgpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let time_code = TimeUnit::Seconds(frame as f64 / config.frames_per_second).into();
        let progress = frame as f64 / frames as f64;

        let mut canvas_texture = texture_factory.borrow_texture(&wgpu);
        let mut blended_texture = texture_factory.borrow_texture(&wgpu);

        let frame_info = FrameInfo {
            time_code,
            progress,
            resolution: config.resolution,
        };

        for clip in project.clips_mut() {
            let output = render_clip(
                clip,
                &wgpu,
                frame_info,
                project_range,
                Mat4::identity(),
                &mut encoder,
                &mut texture_factory,
                &blend_modes,
            );

            if let Some(output) = output {
                // Swap to reuse textures
                core::mem::swap(&mut canvas_texture, &mut blended_texture);

                blend_modes.normal.blend(
                    &wgpu,
                    &mut encoder,
                    output.view(),
                    canvas_texture.view(),
                    blended_texture.view(),
                );

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

        output.publish_frame(&wgpu, encoder, &handler_blended_texture, frame, frame_info);
    }

    handler_texture_factory.return_texture(handler_canvas_texture);
    handler_texture_factory.return_texture(handler_blended_texture);

    #[rustfmt::skip]
    {
        log::info!("Finished render");
        log::info!("Render stats:");
        log::info!("  Factory textures allocated by clips: {}", texture_factory.created_textures());
        log::info!("  Handler textures allocated by renderer: {}", handler_texture_factory.created_textures());

        if texture_factory.created_textures() != texture_factory.available_textures() {
            log::warn!(
                "Not all factory textures have been returned ({} missing)",
                texture_factory.created_textures() - texture_factory.available_textures(),
            );
        }
        
        if handler_texture_factory.created_textures() != handler_texture_factory.available_textures() {
            log::warn!(
                "Not all handler textures have been returned ({} missing)",
                handler_texture_factory.created_textures() - handler_texture_factory.available_textures(),
            );
        }
    };
}
