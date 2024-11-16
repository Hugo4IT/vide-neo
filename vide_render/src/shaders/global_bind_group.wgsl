struct GlobalUniform {
    ortho_matrix: mat4x4<f32>,
};

@group(GROUP) @binding(0) var<uniform> global: GlobalUniform;