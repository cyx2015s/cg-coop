use crate::{core::math::aabb::AABB, scene::world::{BodyType, GameObject}};

pub fn aabb_intersect(a: &AABB, b: &AABB) -> bool {
    a.min.x <= b.max.x && a.max.x >= b.min.x &&
    a.min.y <= b.max.y && a.max.y >= b.min.y &&
    a.min.z <= b.max.z && a.max.z >= b.min.z
}

pub fn apply_gravity(obj: &mut GameObject, gravity: glam::f32::Vec3, dt: f32) {
    if obj.body_type == BodyType::Dynamic { 
        obj.velocity[1] += gravity.y * dt;
    }
}

pub fn predict_position(obj: &GameObject, dt: f32) -> glam::f32::Vec3 {
    let pos = obj.transform.position;
    let vel = glam::f32::Vec3::from_array(obj.velocity);
    pos + vel * dt
}

pub fn resolve_collision(dynamic_body: &mut GameObject, static_body_aabb: &AABB, predicted_pos: glam::f32::Vec3) -> bool {
    let half_size = dynamic_body.mesh.aabb.get_half_extents();
    let dynamic_aabb = AABB {
        min: predicted_pos - half_size,
        max: predicted_pos + half_size,
    };

    if aabb_intersect(&dynamic_aabb, &static_body_aabb) {
        let penetration = static_body_aabb.max.y - dynamic_aabb.min.y;
        dynamic_body.transform.position.y += penetration;
        dynamic_body.velocity[1] = -dynamic_body.velocity[1] * dynamic_body.restitution;
        if dynamic_body.velocity[1].abs() < 0.1 {
            dynamic_body.velocity[1] = 0.0;
        }
        return true;
    }
    false
}