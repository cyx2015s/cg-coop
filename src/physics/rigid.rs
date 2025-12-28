use crate::{
    core::math::transform::{self, Transform}, physics::boundingbox::{AABB, BoundingVolume}, scene::world::BodyType
};

use glam::f32::Vec3;

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

    fn inv_mass(&self) -> f32 { if self.mass().abs() < 0.0001 { f32::INFINITY } else { 1.0 / self.mass() }  }

    fn force(&self) -> [f32; 3];

    fn force_mut(&mut self) -> &mut [f32; 3];

    fn friction(&self) -> f32;

    fn stimulate(&mut self, dt: f32) {
        if !self.is_dynamic() { return; }
        let mass = self.mass();
        let force = Vec3::from_array(self.force());
        let mut vel = Vec3::from_array(self.velocity());
        // 半隐式 Euler：先更新速度，再更新位置
        let acceleration = force / mass;
        vel += acceleration * dt;  // 先更新速度
        self.transform_mut().position += vel * dt; // 再更新位置
        // 写回速度
        let velocity = self.velocity_mut();
        velocity[0] = vel.x;
        velocity[1] = vel.y;
        velocity[2] = vel.z;
        self.force_mut().fill(0.0);
    }

    fn is_dynamic(&self) -> bool {
        self.body_type() == BodyType::Dynamic
    }

    fn is_static(&self) -> bool {
        self.body_type() == BodyType::Static
    }
}
