use vide_common::{config::RenderConfiguration, render::Wgpu, FrameInfo};

use crate::texture_factory::FactoryTexture;

pub trait OutputHandler {
    fn configure(&mut self, wgpu: &Wgpu, config: &RenderConfiguration) -> wgpu::TextureFormat;
    fn publish_frame(
        &mut self,
        wgpu: &Wgpu,
        encoder: wgpu::CommandEncoder,
        texture: &FactoryTexture,
        frame: i64,
        frame_info: FrameInfo,
    );
}
