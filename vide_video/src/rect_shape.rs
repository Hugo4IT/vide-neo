use std::sync::{Once, OnceLock};

use euler::{vec2, vec3, Mat4, Quat, Trs, Vec2};
use vide_animate::AnimatedProperty;
use vide_common::{
    color::Color, config::RenderConfiguration, prelude::TimeCode, render::Wgpu,
    standards::FRAGMENT_COLOR_TARGET, visible_object::VisibleObject,
};
use wgpu::util::DeviceExt;

static RENDERER: OnceLock<RectShapeRenderer> = OnceLock::new();

fn init_renderer(wgpu: &Wgpu, config: &RenderConfiguration) {
    static ONCE: Once = Once::new();

    ONCE.call_once(move || RENDERER.set(RectShapeRenderer::new(wgpu, config)).unwrap());
}

fn renderer<'a>() -> &'a RectShapeRenderer {
    unsafe { RENDERER.get().unwrap_unchecked() }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct RectShapeData {
    matrix: [[f32; 4]; 4],
    color: [f32; 4],
}

unsafe impl bytemuck::Pod for RectShapeData {}
unsafe impl bytemuck::Zeroable for RectShapeData {}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    const fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[derive(Debug)]
pub struct RectShapeRenderer {
    shader_module: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
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

        let vertex_buffer = wgpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("RectShape Renderer Vertex Buffer"),
                contents: bytemuck::cast_slice(&[
                    // Face 1
                    Vertex {
                        position: [-0.5, -0.5],
                        uv: [0.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, -0.5],
                        uv: [1.0, 1.0],
                    },
                    Vertex {
                        position: [-0.5, 0.5],
                        uv: [0.0, 0.0],
                    },
                    // Face 2
                    Vertex {
                        position: [-0.5, 0.5],
                        uv: [0.0, 0.0],
                    },
                    Vertex {
                        position: [0.5, -0.5],
                        uv: [1.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, 0.5],
                        uv: [1.0, 0.0],
                    },
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let pipeline_layout = wgpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("RectShape Renderer Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &wgpu.global_bind_group_layout],
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
                    buffers: &[Vertex::desc()],
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
            vertex_buffer,
            pipeline,
        }
    }
}

#[derive(Debug, Default)]
pub struct RectShapeInternalData {
    transform: Option<Mat4>,
    buffer: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
}

impl Clone for RectShapeInternalData {
    fn clone(&self) -> Self {
        Self {
            transform: self.transform,
            buffer: None,
            bind_group: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RectShape {
    pub position: AnimatedProperty<Vec2>,
    pub rotation: AnimatedProperty<f64>,
    pub size: AnimatedProperty<Vec2>,
    pub pivot: AnimatedProperty<Vec2>,
    pub color: AnimatedProperty<Color>,
    pub internal: RectShapeInternalData,
}

impl RectShape {
    fn to_data(&self, transform: euler::Mat4, time_code: TimeCode) -> RectShapeData {
        let position = self.position.evaluate(time_code);
        let rotation = self.rotation.evaluate(time_code);
        let size = self.size.evaluate(time_code);
        let pivot = self.pivot.evaluate(time_code);
        let color = self.color.evaluate(time_code);

        let matrix = Trs::new(
            vec3!(
                position - vec2!(size.x * (pivot.x - 0.5), size.y * (pivot.y - 0.5)),
                0.0
            ),
            Quat::axis_angle(vec3!(0.0, 0.0, -1.0), rotation.to_radians() as f32),
            vec3!(size, 1.0),
        )
        .matrix();

        RectShapeData {
            matrix: (transform * matrix).into(),
            color: color.into(),
        }
    }
}

impl VisibleObject for RectShape {
    fn init(&mut self, wgpu: &Wgpu, config: &RenderConfiguration) {
        init_renderer(wgpu, config);

        let buffer = wgpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RectShape Buffer"),
            size: size_of::<RectShapeData>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RectShape Bind Group"),
            layout: &renderer().bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        });

        self.internal.buffer = Some(buffer);
        self.internal.bind_group = Some(bind_group);
    }

    fn duration(&self) -> Option<vide_common::prelude::TimeCode> {
        None
    }

    fn set_transform(&mut self, transform: euler::Mat4) {
        self.internal.transform = Some(transform)
    }

    fn update(
        &mut self,
        wgpu: &Wgpu,
        frame_info: &vide_common::FrameInfo,
        local_frame_info: &vide_common::FrameInfo,
    ) {
        let data = self.to_data(
            self.internal.transform.unwrap_or_else(Mat4::identity),
            local_frame_info.time_code,
        );

        let buffer = self
            .internal
            .buffer
            .as_ref()
            .expect("buffer should be set in init()");

        wgpu.queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
    }

    fn render(
        &mut self,
        wgpu: &Wgpu,
        frame_info: &vide_common::FrameInfo,
        local_frame_info: &vide_common::FrameInfo,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::TextureView,
    ) {
        let bind_group = self
            .internal
            .bind_group
            .as_ref()
            .expect("buffer should be set in init()");

        let renderer = renderer();

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("RectShape Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: destination,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&renderer.pipeline);
        pass.set_vertex_buffer(0, renderer.vertex_buffer.slice(..));
        pass.set_bind_group(0, bind_group, &[]);
        pass.set_bind_group(1, &wgpu.global_bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}
