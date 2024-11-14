use euler::{DVec2, DVec3, DVec4, Quat, Vec2, Vec3, Vec4};
use vide_common::transform::Transform;

pub trait Interpolate {
    fn interpolate(a: Self, b: Self, t: f64) -> Self;

    fn interpolate_to(&self, b: Self, t: f64) -> Self
    where
        Self: Sized + Clone,
    {
        Self::interpolate(self.clone(), b, t)
    }
}

macro_rules! impl_interpolate {
    ($typ:ty) => {
        impl Interpolate for $typ {
            fn interpolate(a: Self, b: Self, t: f64) -> Self {
                ((b - a) as f64 * t) as $typ + a
            }
        }
    };
}

impl_interpolate!(u8);
impl_interpolate!(u16);
impl_interpolate!(u32);
impl_interpolate!(u64);
impl_interpolate!(u128);
impl_interpolate!(i8);
impl_interpolate!(i16);
impl_interpolate!(i32);
impl_interpolate!(i64);
impl_interpolate!(i128);
impl_interpolate!(f32);
impl_interpolate!(f64);

impl<A, B> Interpolate for (A, B)
where
    A: Interpolate,
    B: Interpolate,
{
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        (A::interpolate(a.0, b.0, t), B::interpolate(a.1, b.1, t))
    }
}

impl<A, B, C> Interpolate for (A, B, C)
where
    A: Interpolate,
    B: Interpolate,
    C: Interpolate,
{
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        (
            A::interpolate(a.0, b.0, t),
            B::interpolate(a.1, b.1, t),
            C::interpolate(a.2, b.2, t),
        )
    }
}

impl<A, B, C, D> Interpolate for (A, B, C, D)
where
    A: Interpolate,
    B: Interpolate,
    C: Interpolate,
    D: Interpolate,
{
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        (
            A::interpolate(a.0, b.0, t),
            B::interpolate(a.1, b.1, t),
            C::interpolate(a.2, b.2, t),
            D::interpolate(a.3, b.3, t),
        )
    }
}

impl Interpolate for Vec2 {
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self::new(f32::interpolate(a.x, b.x, t), f32::interpolate(a.y, b.y, t))
    }
}

impl Interpolate for Vec3 {
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self::new(
            f32::interpolate(a.x, b.x, t),
            f32::interpolate(a.y, b.y, t),
            f32::interpolate(a.z, b.z, t),
        )
    }
}

impl Interpolate for Vec4 {
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self::new(
            f32::interpolate(a.x, b.x, t),
            f32::interpolate(a.y, b.y, t),
            f32::interpolate(a.z, b.z, t),
            f32::interpolate(a.w, b.w, t),
        )
    }
}

impl Interpolate for DVec2 {
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self::new(f64::interpolate(a.x, b.x, t), f64::interpolate(a.y, b.y, t))
    }
}

impl Interpolate for DVec3 {
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self::new(
            f64::interpolate(a.x, b.x, t),
            f64::interpolate(a.y, b.y, t),
            f64::interpolate(a.z, b.z, t),
        )
    }
}

impl Interpolate for DVec4 {
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self::new(
            f64::interpolate(a.x, b.x, t),
            f64::interpolate(a.y, b.y, t),
            f64::interpolate(a.z, b.z, t),
            f64::interpolate(a.w, b.w, t),
        )
    }
}

impl Interpolate for Quat {
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self::new(
            f32::interpolate(a.x, b.x, t),
            f32::interpolate(a.y, b.y, t),
            f32::interpolate(a.z, b.z, t),
            f32::interpolate(a.s, b.s, t),
        )
    }
}

impl Interpolate for Transform {
    fn interpolate(a: Self, b: Self, t: f64) -> Self {
        Self::from_components(
            a.position().interpolate_to(b.position(), t),
            a.rotation().interpolate_to(b.rotation(), t),
            a.scale().interpolate_to(b.scale(), t),
        )
    }
}
