use std::fmt::Debug;

pub trait EaseSampler: Debug {
    fn sample(&self, t: f64) -> f64;
}

impl<F: Fn(f64) -> f64 + Debug> EaseSampler for F {
    fn sample(&self, t: f64) -> f64 {
        self(t)
    }
}
