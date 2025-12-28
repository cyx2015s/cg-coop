use crate::physics::rigid::{Contact, RigidBody};

pub fn solve_contact(
    a: &mut dyn RigidBody,
    b: &mut dyn RigidBody,
    contact: &Contact,
    dt: f32,
) {
    positional_correction(a, b, contact);
    apply_normal_impulse(a, b, contact, dt);
    apply_friction(a, b, contact, dt);
}

pub fn positional_correction(
    a: &mut dyn RigidBody,
    b: &mut dyn RigidBody,
    contact: &Contact,
) {
    
}

pub fn apply_normal_impulse(
    a: &mut dyn RigidBody,
    b: &mut dyn RigidBody,
    contact: &Contact,
    dt: f32,
) {
    
}

pub fn apply_friction(
    a: &mut dyn RigidBody,
    b: &mut dyn RigidBody,
    contact: &Contact,
    dt: f32,
) {
    
}

// 对特定物体应用重力
pub fn apply_gravity(obj: &mut dyn RigidBody, gravity: glam::f32::Vec3) {
    let mass = obj.mass();
    if obj.is_dynamic() {
        let force = obj.force_mut();
        force[1] = force[1] + gravity.y * mass;
    }
}