use prelude::TimeCode;
use time_code::UnboundedTimecodeRange;
use types::{Resolution, TimeUnit};

pub mod color;
pub mod config;
pub mod prelude;
pub mod render;
pub mod standards;
pub mod time_code;
pub mod transform;
pub mod types;
pub mod visible_object;

#[derive(Debug, Clone, Copy)]
pub struct FrameInfo {
    pub time_code: TimeCode,
    pub progress: f64,
    pub resolution: Resolution,
}

impl FrameInfo {
    pub fn make_local(&self, range: UnboundedTimecodeRange) -> Self {
        let time_code = self.time_code - range.start().unwrap_or_default();
        let progress = time_code.value() as f64
            / range
                .duration()
                .unwrap_or(TimeCode::from(TimeUnit::Seconds(1.0)))
                .value() as f64;

        Self {
            time_code,
            progress,
            resolution: self.resolution,
        }
    }
}
