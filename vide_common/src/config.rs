use crate::types::{FramesPerSecond, Resolution};

pub mod presets {
    use crate::types::{FramesPerSecond, Resolution};

    pub const RESOLUTION_480P_4X3: Resolution = (640, 480);

    pub const RESOLUTION_720P_16X9: Resolution = (1280, 720);

    pub const RESOLUTION_1080P_16X9: Resolution = (1920, 1080);
    pub const RESOLUTION_2K_16X9: Resolution = RESOLUTION_1080P_16X9;

    pub const RESOLUTION_2160P_16X9: Resolution = (3840, 2160);
    pub const RESOLUTION_4K_16X9: Resolution = RESOLUTION_2160P_16X9;

    pub const RESOLUTION_4320P_16X9: Resolution = (7680, 4320);
    pub const RESOLUTION_8K_16X9: Resolution = RESOLUTION_4320P_16X9;

    pub const RESOLUTION_8640P_16X9: Resolution = (15360, 8640);
    pub const RESOLUTION_16K_16X9: Resolution = RESOLUTION_8640P_16X9;

    pub const FPS_24: FramesPerSecond = 24.0;
    pub const FPS_CINEMATIC: FramesPerSecond = FPS_24;
    pub const FPS_30: FramesPerSecond = 30.0;
    pub const FPS_60: FramesPerSecond = 60.0;
    pub const FPS_120: FramesPerSecond = 120.0;
    pub const FPS_240: FramesPerSecond = 240.0;
}

#[derive(Debug, Clone, Copy)]
pub struct RenderConfiguration {
    pub resolution: Resolution,
    pub frames_per_second: FramesPerSecond,
    pub hdr: bool,
}
