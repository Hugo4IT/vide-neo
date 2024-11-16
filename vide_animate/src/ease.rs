use std::fmt::Debug;

pub trait EaseSampler: Debug + EaseSamplerClone {
    fn sample(&self, t: f64) -> f64;
}

pub trait EaseSamplerClone {
    fn clone_sampler(&self) -> Box<dyn EaseSampler>;
}

impl<T: 'static + EaseSampler + Clone> EaseSamplerClone for T {
    fn clone_sampler(&self) -> Box<dyn EaseSampler> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn EaseSampler> {
    fn clone(&self) -> Self {
        self.clone_sampler()
    }
}

impl<F: Fn(f64) -> f64 + Debug + Clone + 'static> EaseSampler for F {
    fn sample(&self, t: f64) -> f64 {
        self(t)
    }
}
