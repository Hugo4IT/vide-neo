use std::ops::{Add, Range, RangeFrom, RangeFull, RangeTo, Sub};

use crate::types::{Frames, FramesPerSecond, Seconds, TimeUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TimeCode {
    value: i64,
}

impl TimeCode {
    const SECOND: i64 = 60_000;
    const MILLISECOND: i64 = Self::SECOND / 1000;

    #[inline]
    pub const fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn seconds(&self) -> Seconds {
        self.value as Seconds / Self::SECOND as Seconds
    }

    pub fn frame(&self, fps: FramesPerSecond) -> Frames {
        (self.seconds() * fps).floor() as Frames
    }

    #[inline]
    pub const fn value(&self) -> i64 {
        self.value
    }
}

impl From<TimeUnit> for TimeCode {
    fn from(value: TimeUnit) -> Self {
        match value {
            TimeUnit::Timecode(value) => Self::new(value),
            TimeUnit::Seconds(value) => Self::new((value * TimeCode::SECOND as Seconds) as i64),
            TimeUnit::Milliseconds(value) => Self::new(value * TimeCode::MILLISECOND),
        }
    }
}

impl Add for TimeCode {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.value() + rhs.value())
    }
}

impl Sub for TimeCode {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.value() - rhs.value())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnboundedTimecodeRange {
    start: Option<TimeCode>,
    end: Option<TimeCode>,
}

impl UnboundedTimecodeRange {
    pub fn new(start: Option<TimeCode>, end: Option<TimeCode>) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> Option<TimeCode> {
        self.start
    }

    pub fn end(&self) -> Option<TimeCode> {
        self.end
    }

    pub fn duration(&self) -> Option<TimeCode> {
        Some(self.end? - self.start?)
    }

    pub fn set_duration(&mut self, duration: TimeCode) {
        self.end = Some(TimeCode::new(
            self.start.map_or(0, |t| t.value()) + duration.value(),
        ));
    }

    pub fn make_absolute(self, parent: UnboundedTimecodeRange) -> Self {
        let start = self.start.unwrap_or_default() + parent.start().unwrap_or_default();

        Self::new(
            Some(start),
            match (self.end, parent.end) {
                (Some(own_end), Some(parent_end)) => {
                    Some(start + (parent_end - parent.start().unwrap_or_default()).min(own_end))
                }
                (Some(own_end), None) => Some(start + own_end),
                (None, Some(parent_end)) => {
                    Some(start + (parent_end - parent.start().unwrap_or_default()))
                }
                _ => None,
            },
        )
    }

    pub fn contains(&self, time_code: TimeCode) -> bool {
        match (self.start(), self.end()) {
            (Some(start), Some(end)) => (start..end).contains(&time_code),
            (Some(start), None) => time_code >= start,
            (None, Some(end)) => time_code < end,
            _ => true,
        }
    }
}

impl<T> From<Range<T>> for UnboundedTimecodeRange
where
    TimeUnit: From<T>,
{
    fn from(value: Range<T>) -> Self {
        Self::new(
            Some(TimeUnit::from(value.start).into()),
            Some(TimeUnit::from(value.end).into()),
        )
    }
}

impl<T> From<RangeTo<T>> for UnboundedTimecodeRange
where
    TimeUnit: From<T>,
{
    fn from(value: RangeTo<T>) -> Self {
        Self::new(None, Some(TimeUnit::from(value.end).into()))
    }
}

impl<T> From<RangeFrom<T>> for UnboundedTimecodeRange
where
    TimeUnit: From<T>,
{
    fn from(value: RangeFrom<T>) -> Self {
        Self::new(Some(TimeUnit::from(value.start).into()), None)
    }
}

impl From<RangeFull> for UnboundedTimecodeRange {
    fn from(_: RangeFull) -> Self {
        Self::new(None, None)
    }
}

impl<T> From<T> for UnboundedTimecodeRange
where
    TimeUnit: From<T>,
{
    fn from(value: T) -> Self {
        Self::new(Some(TimeUnit::from(value).into()), None)
    }
}
