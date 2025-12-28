use crate::{
    core::math::transform::Transform, physics::boundingbox::{AABB, BoundingVolume}, scene::world::BodyType
};

#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub normal: glam::f32::Vec3,
    pub penetration: f32,
}

pub trait RigidBody {
    fn transform(&self) -> &Transform;
    fn transform_mut(&mut self) -> &mut Transform;

    fn velocity(&self) -> [f32; 3];
    fn velocity_mut(&mut self) -> &mut [f32; 3];

    fn body_type(&self) -> BodyType;

    fn restitution(&self) -> f32;

    fn bounding_volume(&self) -> BoundingVolume;

    fn mass(&self) -> f32;

    fn force(&self) -> [f32; 3];

    fn force_mut(&mut self) -> &mut [f32; 3];

    fn friction(&self) -> [f32; 3];

    fn update_velocity(&mut self, dt: f32) {
        let mass = self.mass();
        let force = self.force();
        let friction = self.friction();
        let vel = self.velocity_mut();
        let v_x = vel[0];
        let v_z = vel[2];

        vel[0] += (force[0] - friction[0] * v_x * mass) / mass * dt;
        vel[2] += (force[2] - friction[2] * v_z * mass) / mass * dt;

        if force[0].abs() < 0.01 && vel[0].abs() < 0.1 {
            vel[0] = 0.0;
        }

        if force[2].abs() < 0.01 && vel[2].abs() < 0.1 {
            vel[2] = 0.0;
        }
    }

        fn is_dynamic(&self) -> bool {
        self.body_type() == BodyType::Dynamic
    }

    fn is_static(&self) -> bool {
        self.body_type() == BodyType::Static
    }
}
