use crate::physics::{boundingbox::{AABB, BoundingVolume}, collision::narrow::{aabb_vs_aabb, aabb_vs_sphere, sphere_vs_sphere}, rigid::{Contact, RigidBody}};

pub fn collide(a: &BoundingVolume, b: &BoundingVolume) -> Option<Contact> {
    match (a, b) {
        (BoundingVolume::AABB(a), BoundingVolume::AABB(b)) => aabb_vs_aabb(a, b),
        (BoundingVolume::AABB(a), BoundingVolume::Sphere(b)) => aabb_vs_sphere(a, b),
        (BoundingVolume::Sphere(a), BoundingVolume::AABB(b)) => {
            aabb_vs_sphere(b, a).map(|mut c| {
                c.normal = -c.normal;
                c
            })
        },
        (BoundingVolume::Sphere(a), BoundingVolume::Sphere(b)) => sphere_vs_sphere(a, b),
        _ => None,
    }
}

// 检测两个AABB是否相交
pub fn aabb_overlap(a: &AABB, b: &AABB) -> bool {
    a.min.x <= b.max.x &&
    a.max.x >= b.min.x &&
    a.min.y <= b.max.y &&
    a.max.y >= b.min.y &&
    a.min.z <= b.max.z &&
    a.max.z >= b.min.z
}



// 预测物体下一帧的位置
pub fn predict_position(obj: &dyn RigidBody, dt: f32) -> glam::f32::Vec3 {
    let pos = obj.transform().position;
    let vel = glam::f32::Vec3::from_array(obj.velocity());
    pos + vel * dt
}

// 解决碰撞：目前只有y轴的碰撞
pub fn resolve_collision(a: &mut dyn RigidBody, b: &mut dyn RigidBody, dt: f32) -> bool {
    if b.is_dynamic() {
        return false;
    }
    let half_size_a = a.aabb().get_half_extents();
    let predicted_pos_a = predict_position(a, dt);
    let half_size_b = b.aabb().get_half_extents();
    let predicted_pos_b = predict_position(b, dt);
    let a_aabb = AABB {
        min: predicted_pos_a - half_size_a,
        max: predicted_pos_a + half_size_a,
    };

    let b_aabb = AABB {
        min: predicted_pos_b - half_size_b,
        max: predicted_pos_b + half_size_b,
    };

    if aabb_overlap(&a_aabb, &b_aabb) {
        let penetration = b_aabb.max.y - a_aabb.min.y;
        a.transform_mut().position.y += penetration;
        a.velocity_mut()[1] = -a.velocity_mut()[1] * a.restitution();
        if a.velocity()[1].abs() < 0.1 {
            a.velocity_mut()[1] = 0.0;
        }
        return true;
    }
    false
}
