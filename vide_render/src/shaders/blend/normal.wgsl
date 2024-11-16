@group(0) @binding(0) var a_texture: texture_2d<f32>;
@group(0) @binding(1) var b_texture: texture_2d<f32>;
@group(0) @binding(2) var source_sampler: sampler;

@fragment
fn fragment_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    let a = textureSample(a_texture, source_sampler, tex_coords);
    let b = textureSample(b_texture, source_sampler, tex_coords);

    // A over B alpha blending
    let alpha = a.a + b.a * (1.0 - a.a);
    let color = (a.rgb * a.a + b.rgb * b.a * (1.0 - a.a)) / alpha;

    return vec4<f32>(color, alpha);
}