#![feature(const_fn_floating_point_arithmetic)]
#![feature(array_windows)]

use std::fmt::Debug;

use ease::EaseSampler;
use prelude::Interpolate;
use vide_common::prelude::TimeCode;

pub mod cubic_bezier;
pub mod ease;
pub mod interpolate;
pub mod prelude;

#[derive(Debug)]
pub struct Keyframe<T: Interpolate + Debug + Clone> {
    easing: Option<Box<dyn EaseSampler>>,
    time_code: TimeCode,
    value: T,
}

impl<T: Interpolate + Debug + Clone> Keyframe<T> {
    pub fn evaluate(&self, previous: &Keyframe<T>, time_code: TimeCode) -> T {
        let mut t = (time_code - previous.time_code).value() as f64
            / (self.time_code - previous.time_code).value() as f64;

        if let Some(easing) = self.easing.as_ref() {
            t = easing.sample(t);
        }

        previous.value.interpolate_to(self.value.clone(), t)
    }
}

#[derive(Debug)]
pub struct AnimatedProperty<T: Interpolate + Debug + Clone> {
    default: T,
    keyframes: Vec<Keyframe<T>>,
}

impl<T: Interpolate + Debug + Clone> AnimatedProperty<T> {
    pub fn with_default(default: T) -> Self {
        Self {
            default,
            keyframes: Vec::new(),
        }
    }

    pub fn builder_with_default(default: T) -> AnimatedPropertyBuilder<T> {
        AnimatedPropertyBuilder::with_default(default)
    }

    pub fn push_keyframe(&mut self, keyframe: Keyframe<T>) {
        self.keyframes.push(keyframe);
    }

    pub fn evaluate(&self, time_code: TimeCode) -> T {
        if let Some(keyframe) = self.keyframes.first() {
            if keyframe.time_code >= time_code {
                return keyframe.evaluate(
                    &Keyframe {
                        easing: None,
                        time_code: TimeCode::new(0),
                        value: self.default.clone(),
                    },
                    time_code,
                );
            }
        } else {
            return self.default.clone();
        }

        for [previous, current] in self.keyframes.array_windows() {
            if current.time_code >= time_code {
                return current.evaluate(previous, time_code);
            }
        }

        // All keyframes passed, return last one
        if let Some(keyframe) = self.keyframes.last() {
            return keyframe.value.clone();
        }

        self.default.clone()
    }
}

impl<T: Interpolate + Debug + Clone + Default> AnimatedProperty<T> {
    pub fn new() -> Self {
        Self {
            default: T::default(),
            keyframes: Vec::new(),
        }
    }

    pub fn builder() -> AnimatedPropertyBuilder<T> {
        AnimatedPropertyBuilder::new()
    }
}

impl<T: Interpolate + Debug + Clone + Default> Default for AnimatedProperty<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum KeyframeTiming<T: Into<TimeCode>> {
    Abs(T),
    Rel(T),
}

#[derive(Debug)]
pub struct AnimatedPropertyBuilder<T: Interpolate + Debug + Clone> {
    animation: AnimatedProperty<T>,
}

impl<T: Interpolate + Debug + Clone> AnimatedPropertyBuilder<T> {
    pub fn with_default(default: T) -> Self {
        Self {
            animation: AnimatedProperty::with_default(default),
        }
    }

    pub fn keyframe(
        mut self,
        at: KeyframeTiming<impl Into<TimeCode>>,
        value: impl Into<T>,
        easing: Option<impl EaseSampler + 'static>,
    ) -> Self {
        let time_code = match at {
            KeyframeTiming::Abs(t) => t.into(),
            KeyframeTiming::Rel(t) => {
                self.animation
                    .keyframes
                    .last()
                    .map(|k| k.time_code)
                    .unwrap_or(TimeCode::new(0))
                    + t.into()
            }
        };

        if time_code.value() == 0 {
            self.animation.default = value.into();

            self
        } else {
            self.animation.push_keyframe(Keyframe {
                // Does not work with regular map
                easing: match easing {
                    Some(easing) => Some(Box::new(easing)),
                    None => None,
                },
                time_code,
                value: value.into(),
            });

            self
        }
    }

    pub fn hold(mut self, duration: impl Into<TimeCode>) -> Self {
        let duration = duration.into();

        let (value, offset) = if let Some(last) = self.animation.keyframes.last() {
            (last.value.clone(), last.time_code)
        } else {
            (self.animation.default.clone(), TimeCode::new(0))
        };

        self.animation.push_keyframe(Keyframe {
            easing: None,
            time_code: offset + duration,
            value,
        });

        self
    }

    pub fn build(self) -> AnimatedProperty<T> {
        self.animation
    }
}

impl<T: Interpolate + Debug + Clone + Default> AnimatedPropertyBuilder<T> {
    pub fn new() -> Self {
        Self {
            animation: AnimatedProperty::new(),
        }
    }
}

impl<T: Interpolate + Debug + Clone + Default> Default for AnimatedPropertyBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn value<T: Interpolate + Debug + Clone>(value: impl Into<T>) -> AnimatedProperty<T> {
    AnimatedProperty::with_default(value.into())
}

pub fn animated<T: Interpolate + Debug + Clone>(
    initial_value: impl Into<T>,
) -> AnimatedPropertyBuilder<T> {
    AnimatedProperty::builder_with_default(initial_value.into())
}
