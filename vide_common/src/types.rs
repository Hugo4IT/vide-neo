pub type Resolution = (u64, u64);
pub type Frames = i64;
pub type FramesPerSecond = f64;
pub type Seconds = f64;
pub type Milliseconds = i64;

#[derive(Debug, Clone, Copy)]
pub enum TimeUnit {
    Timecode(i64),
    Seconds(Seconds),
    Milliseconds(Milliseconds),
}

impl From<f32> for TimeUnit {
    fn from(value: f32) -> Self {
        Self::Seconds(value as Seconds)
    }
}

impl From<Seconds> for TimeUnit {
    fn from(value: Seconds) -> Self {
        Self::Seconds(value)
    }
}
