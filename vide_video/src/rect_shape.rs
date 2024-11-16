use std::sync::{Once, OnceLock};

use euler::{Vec2, Vec4};
use vide_animate::AnimatedProperty;
use vide_common::{
    color::Color, config::RenderConfiguration, render::Wgpu, standards::FRAGMENT_COLOR_TARGET,
    visible_object::VisibleObject,
};

static RENDERER: OnceLock<RectShapeRenderer> = OnceLock::new();

fn init_renderer(wgpu: &Wgpu, config: &RenderConfiguration) {
    static ONCE: Once = Once::new();

    ONCE.call_once(move || RENDERER.set(RectShapeRenderer::new(wgpu, config)).unwrap());
}

fn renderer<'a>() -> &'a RectShapeRenderer {
    unsafe { RENDERER.get().unwrap_unchecked() }
}

#[derive(Debug)]
pub struct RectShapeRenderer {
    shader_module: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl RectShapeRenderer {
    pub fn new(wgpu: &Wgpu, config: &RenderConfiguration) -> Self {
        let shader_module = wgpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RectShape Renderer Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/rect_shape.wgsl").into()),
            });

        let bind_group_layout =
            wgpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("RectShape Renderer Bind Group Layout"),
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

        let pipeline_layout = wgpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("RectShape Renderer Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = wgpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Rect Shape Renderer Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: None,
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: None,
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: FRAGMENT_COLOR_TARGET,
                        blend: None,
                        write_mask: wgpu::ColorWrites::all(),
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        Self {
            shader_module,
            bind_group_layout,
            pipeline,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RectShape {
    pub position: AnimatedProperty<Vec2>,
    pub size: AnimatedProperty<Vec2>,
    pub pivot: AnimatedProperty<Vec2>,
    pub color: AnimatedProperty<Color>,
}

impl VisibleObject for RectShape {
    fn init(&mut self, wgpu: &Wgpu, config: &RenderConfiguration) {
        todo!()
    }

    fn duration(&self) -> Option<vide_common::prelude::TimeCode> {
        todo!()
    }

    fn set_transform(&mut self, transform: euler::Mat4) {
        todo!()
    }

    fn update(
        &mut self,
        wgpu: &Wgpu,
        frame_info: &vide_common::FrameInfo,
        local_frame_info: &vide_common::FrameInfo,
    ) {
        todo!()
    }

    fn render(
        &mut self,
        wgpu: &Wgpu,
        frame_info: &vide_common::FrameInfo,
        local_frame_info: &vide_common::FrameInfo,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::TextureView,
    ) {
        todo!()
    }
}
