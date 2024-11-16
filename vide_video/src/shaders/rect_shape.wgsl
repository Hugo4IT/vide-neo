struct TransformUniform {
    transform_matrix: mat4x4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> transform_uniform: TransformUniform;

struct GlobalUniform {
    ortho_matrix: mat4x4<f32>,
};

@group(1) @binding(0) var<uniform> global: GlobalUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let transform_matrix = transform_uniform.transform_matrix;
    let color = transform_uniform.color;

    var out: VertexOutput;
    out.color = color;
    out.clip_position = global.ortho_matrix * transform_matrix * vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}