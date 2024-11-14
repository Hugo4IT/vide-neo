use euler::{Mat4, Quat, Trs, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,

    matrix: Mat4,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),

            matrix: Mat4::identity(),
        }
    }

    pub fn from_components(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let mut result = Self {
            position,
            rotation,
            scale,

            matrix: Mat4::identity(),
        };

        result.compute_matrix();

        result
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;

        self.compute_matrix();
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;

        self.compute_matrix();
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;

        self.compute_matrix();
    }

    /// # Safety
    ///
    /// Caller must ensure that `.compute_matrix` is called before the transform
    /// is used
    pub unsafe fn set_position_without_update(&mut self, position: Vec3) {
        self.position = position;
    }

    /// # Safety
    ///
    /// Caller must ensure that `.compute_matrix` is called before the transform
    /// is used
    pub unsafe fn set_rotation_without_update(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    /// # Safety
    ///
    /// Caller must ensure that `.compute_matrix` is called before the transform
    /// is used
    pub unsafe fn set_scale_without_update(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    pub fn compute_matrix(&mut self) {
        self.matrix = Trs::new(self.position, self.rotation, self.scale).matrix();
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
