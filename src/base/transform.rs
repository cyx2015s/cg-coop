pub struct Transform {
    pub position: glam::f32::Vec3,
    pub rotation: glam::f32::Quat,
    pub scale: glam::f32::Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glam::f32::Vec3::ZERO,
            rotation: glam::f32::Quat::IDENTITY,
            scale: glam::f32::Vec3::ONE,
        }
    }

    pub fn look_at(&mut self, target: glam::f32::Vec3, up: glam::f32::Vec3) {
        self.rotation = glam::f32::Quat::look_at_rh(self.position, target, up);
    }

    pub fn get_up(&self) -> glam::f32::Vec3 {
        self.rotation * glam::f32::Vec3::Y
    }

    pub fn get_right(&self) -> glam::f32::Vec3 {
        self.rotation * glam::f32::Vec3::X
    }

    pub fn get_forward(&self) -> glam::f32::Vec3 {
        self.rotation * glam::f32::Vec3::NEG_Z
    }

    pub fn get_matrix(&self) -> glam::f32::Mat4 {
        glam::f32::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}
