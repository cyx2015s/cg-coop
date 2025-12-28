#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: glam::f32::Vec3,
    pub rotation: glam::f32::Quat,
    pub scale: glam::f32::Vec3,
}

impl Transform {
    pub fn new(
        position: glam::f32::Vec3,
        rotation: glam::f32::Quat,
        scale: glam::f32::Vec3,
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn default() -> Self {
        Transform::new(
            glam::f32::Vec3::ZERO,
            glam::f32::Quat::IDENTITY,
            glam::f32::Vec3::ONE,
        )
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

pub fn look_at_rh(
    eye: glam::f32::Vec3,
    center: glam::f32::Vec3,
    up: glam::f32::Vec3,
) -> glam::f32::Quat {
    look_to_rh(center - eye, up)
}
pub fn look_to_rh(dir: glam::f32::Vec3, up: glam::f32::Vec3) -> glam::f32::Quat {
    let f = dir.normalize(); 
    let u0 = up.normalize(); 

    let r = f.cross(u0).normalize();

    let u = r.cross(f);

    let rot = glam::f32::Mat3::from_cols(
        r,  // +X
        u,  // +Y
        -f, // +Z 
    );

    glam::f32::Quat::from_mat3(&rot)
}
