use vide_common::{config::RenderConfiguration, render::Wgpu};

pub trait OutputHandler {
    fn configure(&mut self, wgpu: &Wgpu, config: &RenderConfiguration) -> wgpu::TextureFormat;
    fn publish_frame(&mut self, wgpu: &Wgpu, texture: &wgpu::Texture);
}
