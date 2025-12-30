use crate::{physics::rigid::{Contact, RigidBody}, scene::{World, world::BodyHandle}};


use glam::f32::Vec3;
pub fn solve_contact(
    a: &mut dyn RigidBody,
    b: &mut dyn RigidBody,
    contact: &Contact,
    dt: f32,
) {
    positional_correction(a, b, contact);
    update_vel(a, b, contact, dt);
}

pub fn positional_correction(
    a: &mut dyn RigidBody,
    b: &mut dyn RigidBody,
    contact: &Contact,
) {
    // Baumgarte稳定性修正
    let percent = 0.8;
    let slop = 0.01;
    let correction_mag = ((contact.penetration - slop).max(0.0) / (a.inv_mass() + b.inv_mass())) * percent;
    let correction = contact.normal * correction_mag;


    if b.is_dynamic() {
        let inv_mass = a.inv_mass();
        a.transform_mut().position -= correction * inv_mass;
        let inv_mass = b.inv_mass();
        b.transform_mut().position += correction * inv_mass;
    } else {
        a.transform_mut().position -= contact.normal * contact.penetration * 1.1;
    }
}

pub fn update_vel(
    a: &mut dyn RigidBody,
    b: &mut dyn RigidBody,
    contact: &Contact,
    dt: f32,
) {
    let relative_velocity = Vec3::from_array(a.velocity()) - Vec3::from_array(b.velocity());
    let friction_a = a.friction();
    // let friction_b = b.friction();
    let mass_a = a.mass();
    let mass_b = b.mass();
    let restituion = a.restitution();
    let vel_along_normal = relative_velocity.dot(contact.normal);
    let vel_along_tangent = relative_velocity - contact.normal * vel_along_normal;
    let mut frac_a: f32 = 1.0;
    let mut frac_b: f32 = 0.0;
    if b.is_dynamic(){
        frac_a = (mass_a - restituion * mass_b) / (mass_a + mass_b);
        frac_b = (mass_a + restituion * mass_a) / (mass_a + mass_b);    
    } else {
        frac_a = -restituion;
        frac_b = 0.0;
    }
    let vel_b = Vec3::from_array(b.velocity());
    let vel_a_new_normal = vel_b + frac_a * vel_along_normal * contact.normal;
    let vel_a_new_tangent = vel_along_tangent - vel_along_tangent.normalize_or_zero() * friction_a * vel_along_tangent.length() * dt;
    let vel_b_new = (vel_b + frac_b * vel_along_normal * contact.normal).to_array();
    let vel_a_new = (vel_a_new_normal + vel_a_new_tangent).to_array();
    for i in 0..3 {
        if vel_a_new[i].abs() < 0.1 { a.velocity_mut()[i] = 0.0; } else { a.velocity_mut()[i] = vel_a_new[i]; }
        if vel_b_new[i].abs() < 0.1 { b.velocity_mut()[i] = 0.0; } else { b.velocity_mut()[i] = vel_b_new[i]; }
    }
}

// 对特定物体应用重力
pub fn apply_gravity(obj: &mut dyn RigidBody, gravity: [f32; 3]) {
    let mass = obj.mass();
    if obj.is_dynamic() {
        let force = obj.force_mut();
        force[1] += gravity[1] * mass;
    }
}

pub fn stimulate_step(handle: BodyHandle, world: &mut World, dt: f32) {
    match handle {
        BodyHandle::Object(idx) => {
            let obj = &mut world.objects[idx];
            apply_gravity(obj, world.gravity);
            obj.stimulate(dt);
        }
        BodyHandle::Camera(idx) => {
            let cam = &mut world.cameras[idx];
            apply_gravity(cam, world.gravity);
            cam.stimulate(dt);
        }
    }
}