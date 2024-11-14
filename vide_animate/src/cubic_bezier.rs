use crate::ease::EaseSampler;

// Mostly ported from https://chromium.googlesource.com/chromium/blink/+/master/Source/platform/animation/UnitBezier.h
#[derive(Debug, Clone, Copy)]
pub struct CubicBezier {
    cx: f64,
    bx: f64,
    ax: f64,

    cy: f64,
    by: f64,
    ay: f64,

    gradient_start: f64,
    gradient_end: f64,
}

impl CubicBezier {
    pub const fn new(point1_x: f64, point1_y: f64, point2_x: f64, point2_y: f64) -> Self {
        let cx = 3.0 * point1_x;
        let bx = 3.0 * (point2_x - point1_x) - cx;
        let ax = 1.0 - cx - bx;

        let cy = 3.0 * point1_y;
        let by = 3.0 * (point2_y - point1_y) - cy;
        let ay = 1.0 - cy - by;

        let gradient_start = if point1_x > 0.0 {
            point1_y / point1_x
        } else if point1_y == 0.0 && point2_x > 0.0 {
            point2_y / point2_x
        } else {
            0.0
        };

        let gradient_end = if point2_x < 1.0 {
            (point2_y - 1.0) / (point2_x - 1.0)
        } else if point2_x == 1.0 && point1_x < 1.0 {
            (point1_y - 1.0) / (point1_x - 1.0)
        } else {
            0.0
        };

        Self {
            cx,
            bx,
            ax,
            cy,
            by,
            ay,
            gradient_start,
            gradient_end,
        }
    }

    pub const fn sample_curve_x(&self, t: f64) -> f64 {
        ((self.ax * t + self.bx) * t + self.cx) * t
    }

    pub const fn sample_curve_y(&self, t: f64) -> f64 {
        ((self.ay * t + self.by) * t + self.cy) * t
    }

    pub const fn sample_curve_derivative_x(&self, t: f64) -> f64 {
        (3.0 * self.ax * t + 2.0 * self.bx) * t + self.cx
    }

    pub fn solve_curve_x(&self, x: f64, epsilon: f64) -> f64 {
        debug_assert!(x >= 0.0);
        debug_assert!(x <= 1.0);

        // First try fast method

        let mut t2 = x;
        for _ in 0..8 {
            let x2 = self.sample_curve_x(t2) - x;

            if x2.abs() < epsilon {
                return t2;
            }

            let d2 = self.sample_curve_derivative_x(t2);

            if d2.abs() < 1e-6 {
                break;
            }

            t2 -= x2 / d2;
        }

        // Fallback to slower method

        let mut t0 = 0.0;
        let mut t1 = 1.0;
        t2 = x;

        while t0 < t1 {
            let x2 = self.sample_curve_x(t2);

            if (x2 - x).abs() < epsilon {
                return t2;
            }

            if x > x2 {
                t0 = t2;
            } else {
                t1 = t2;
            }

            t2 = (t1 - t0) * 0.5 + t0;
        }

        // Failed
        t2
    }

    pub fn solve(&self, x: f64, epsilon: f64) -> f64 {
        if x < 0.0 {
            self.gradient_start * x
        } else if x > 1.0 {
            1.0 + self.gradient_end * (x - 1.0)
        } else {
            self.sample_curve_y(self.solve_curve_x(x, epsilon))
        }
    }
}

impl EaseSampler for CubicBezier {
    fn sample(&self, t: f64) -> f64 {
        self.solve(t, 1e-6)
    }
}

pub mod consts {
    use super::CubicBezier;

    pub const EASE_IN_BACK: CubicBezier = CubicBezier::new(0.36, 0.0, 0.66, -0.56);
    pub const EASE_IN_CIRC: CubicBezier = CubicBezier::new(0.55, 0.0, 1.0, 0.45);
    pub const EASE_IN_CUBIC: CubicBezier = CubicBezier::new(0.32, 0.0, 0.67, 0.0);
    pub const EASE_IN_EXPO: CubicBezier = CubicBezier::new(0.7, 0.0, 0.84, 0.0);
    pub const EASE_IN_OUT_BACK: CubicBezier = CubicBezier::new(0.68, -0.6, 0.32, 1.6);
    pub const EASE_IN_OUT_CIRC: CubicBezier = CubicBezier::new(0.85, 0.0, 0.15, 1.0);
    pub const EASE_IN_OUT_CUBIC: CubicBezier = CubicBezier::new(0.65, 0.0, 0.35, 1.0);
    pub const EASE_IN_OUT_EXPO: CubicBezier = CubicBezier::new(0.87, 0.0, 0.13, 1.0);
    pub const EASE_IN_OUT_QUAD: CubicBezier = CubicBezier::new(0.45, 0.0, 0.55, 1.0);
    pub const EASE_IN_OUT_QUART: CubicBezier = CubicBezier::new(0.76, 0.0, 0.24, 1.0);
    pub const EASE_IN_OUT_QUINT: CubicBezier = CubicBezier::new(0.83, 0.0, 0.17, 1.0);
    pub const EASE_IN_OUT_SINE: CubicBezier = CubicBezier::new(0.37, 0.0, 0.63, 1.0);
    pub const EASE_IN_QUAD: CubicBezier = CubicBezier::new(0.11, 0.0, 0.5, 0.0);
    pub const EASE_IN_QUART: CubicBezier = CubicBezier::new(0.5, 0.0, 0.75, 0.0);
    pub const EASE_IN_QUINT: CubicBezier = CubicBezier::new(0.64, 0.0, 0.78, 0.0);
    pub const EASE_IN_SINE: CubicBezier = CubicBezier::new(0.12, 0.0, 0.39, 0.0);
    pub const EASE_OUT_BACK: CubicBezier = CubicBezier::new(0.34, 1.56, 0.64, 1.0);
    pub const EASE_OUT_CIRC: CubicBezier = CubicBezier::new(0.0, 0.55, 0.45, 1.0);
    pub const EASE_OUT_CUBIC: CubicBezier = CubicBezier::new(0.33, 1.0, 0.68, 1.0);
    pub const EASE_OUT_EXPO: CubicBezier = CubicBezier::new(0.16, 1.0, 0.3, 1.0);
    pub const EASE_OUT_QUAD: CubicBezier = CubicBezier::new(0.5, 1.0, 0.89, 1.0);
    pub const EASE_OUT_QUART: CubicBezier = CubicBezier::new(0.25, 1.0, 0.5, 1.0);
    pub const EASE_OUT_QUINT: CubicBezier = CubicBezier::new(0.22, 1.0, 0.36, 1.0);
    pub const EASE_OUT_SINE: CubicBezier = CubicBezier::new(0.61, 1.0, 0.88, 1.0);
}
