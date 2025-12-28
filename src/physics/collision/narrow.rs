use std::fs::DirBuilder;
use glam::f32::{Vec3};
use crate::{physics::{boundingbox::{AABB, SphereBox}, collision::board::aabb_overlap, rigid::Contact}};

pub fn aabb_vs_aabb(a: &AABB, b: &AABB) -> Option<Contact>{
    let a_center = a.center();
    let b_center = b.center();

    let a_half = a.get_half_extents();
    let b_half = b.get_half_extents();

    let delta = b_center - a_center;

    let overlap = a_half + b_half - delta.abs();

    if overlap.x <= 0.0 || overlap.y <= 0.0 || overlap.z <= 0.0 {
        return None;
    }

    // 找最小穿透轴
    let (penetration, normal) = if overlap.x < overlap.y && overlap.x < overlap.z {
        (overlap.x, Vec3::new(delta.x.signum(), 0.0, 0.0))
    } else if overlap.y < overlap.z {
        (overlap.y, Vec3::new(0.0, delta.y.signum(), 0.0))
    } else {
        (overlap.z, Vec3::new(0.0, 0.0, delta.z.signum()))
    };

    Some(Contact {
        normal,
        penetration,
    })
}

pub fn sphere_vs_sphere(a: &SphereBox, b: &SphereBox)-> Option<Contact>{ 
    let delta = b.center - a.center;
    let dist2 = delta.length_squared();
    let r = a.radius + b.radius;

    if dist2 >= r * r {
        return None;
    }

    let dist = dist2.sqrt();
    let penetration = r - dist;

    let normal = if dist > 0.0 {
        delta / dist
    } else {
        glam::f32::Vec3::X
    };

    Some(Contact {
        normal,
        penetration,
    })
}

pub fn aabb_vs_sphere(aabb: &AABB, sphere: &SphereBox) -> Option<Contact>{
    let closest = sphere.center.clamp(aabb.min,aabb.max);
    let delta = sphere.center - closest;

    let dist2 = delta.length_squared();
    if dist2 >= sphere.radius * sphere.radius {
        return None;
    }

    let dist = dist2.sqrt();
    let penetration = sphere.radius - dist;

    let normal = if dist > 0.0 {
        delta / dist
    } else {
        (sphere.center - aabb.center()).normalize_or_zero()
    };

    Some(Contact {
        normal,
        penetration,
    })
}