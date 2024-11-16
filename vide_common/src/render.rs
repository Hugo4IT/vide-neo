#[derive(Debug)]
pub struct Wgpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalUniform {
    pub ortho_matrix: [[f32; 4]; 4],
}

unsafe impl bytemuck::Pod for GlobalUniform {}
unsafe impl bytemuck::Zeroable for GlobalUniform {}
