use crate::physics::{boundingbox::{BoundingVolume}, collision::narrow::{aabb_vs_aabb, aabb_vs_sphere, sphere_vs_sphere}, rigid::{Contact}};

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
