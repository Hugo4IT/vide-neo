use std::fmt::Debug;

use crate::{config::RenderConfiguration, render::Wgpu, time_code::TimeCode, FrameInfo};
use euler::Mat4;

pub trait VisibleObject: Debug {
    fn init(&mut self, wgpu: &Wgpu, config: &RenderConfiguration);
    fn duration(&self) -> Option<TimeCode>;
    fn set_transform(&mut self, transform: Mat4);
    fn update(&mut self, wgpu: &Wgpu, frame_info: &FrameInfo, local_frame_info: &FrameInfo);
    fn render(
        &mut self,
        wgpu: &Wgpu,
        frame_info: &FrameInfo,
        local_frame_info: &FrameInfo,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::TextureView,
    );
}
