use clip::Clip;
use vide_common::{
    prelude::TimeCode,
    types::{Frames, FramesPerSecond, TimeUnit},
};

pub mod clip;
pub mod prelude;

#[derive(Debug)]
pub struct Project {
    clips: Vec<Clip>,
}

impl Project {
    pub fn new() -> Self {
        Self { clips: Vec::new() }
    }

    pub fn clips(&self) -> &[Clip] {
        &self.clips
    }

    pub fn clips_mut(&mut self) -> &mut [Clip] {
        &mut self.clips
    }

    pub fn add_clip(&mut self, clip: Clip) {
        self.clips.push(clip);
    }

    pub fn duration(&self) -> TimeCode {
        self.clips
            .iter()
            .filter_map(|clip| clip.range().end())
            .max()
            .unwrap_or(TimeUnit::Seconds(5.0).into())
    }

    pub fn frame_count(&self, frames_per_second: FramesPerSecond) -> Frames {
        (self.duration().seconds() * frames_per_second) as Frames
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}
