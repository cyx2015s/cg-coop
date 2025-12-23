use glam::f32::Vec3;
use super::ray::Ray;
use super::aabb::AABB;

pub struct PrimitiveInfo {
    pub pid: usize,
    pub box: AABB,
    pub centroid: Vec3,
}

pub enum Primitive <'a>{
    Sphere(Sphere),
    Triangle(Triangle<'a>),
}

pub struct  Idx {
    pub shapeIdx: usize,
    pub materialIdx: usize,
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub idx: Idx,
}

pub struct Triangle <'a>{
    pub v: [usize; 3],
    pub vertex: &'a [Vec3],
    pub idx: Idx,
}

impl PrimitiveInfo {
    pub fn new(pid: usize, box: AABB) -> Self{
        Self{
            pid,
            box,
            centroid: (box.min + box.max) / 2.0,
        }
    }
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self{
        Self{
            center,
            radius,
        }
    }

    pub fn getAABB(&self) -> AABB{
        AABB::new(
            self.center - Vec3::splat(self.radius),
            self.center + Vec3::splat(self.radius),
        )
    }

    pub fn intersect(&self, ray: &Ray) -> bool{
        let oc = ray.o - self.center;
        let distance = r.d.cross(oc).length_squared();
        return (distance < self.radius * self.radius);
    }
}



impl <'a> Triangle <'a>{
    pub fn new(vi1: usize, vi2: usize, vi3: usize, vertex: &'a [Vec3]) -> Self{
        Self{
            [vi1, vi2, vi3],
            vertex,
        }
    }

    pub fn intersect(&self, ray: &mut ray::Ray) -> bool{
        let o = &ray.o;
        let dir = &ray.d;

        let v0 = &self.vertex[self.v[0]];
        let v1 = &self.vertex[self.v[1]];
        let v2 = &self.vertex[self.v[2]];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;


        let n = edge1.cross(edge2).normalize();

        if dir.dot(n).abs() < 1e-6f32 {
            return false;
        }

        let d = -n.dot(v0);
        let tHit = (d - n.dot(o)) / n.dot(dir);

        if tHit < 0.0 || tHit >= ray.tMax {
            return false;
        }

        let q = o + tHit * dir;
        let area = n.dot(edge1.cross(edge2));
        let alpha = n.dot((v2 - v1).cross(q - v1)) / area;
        let beta = n.dot((q - v0).cross(v2 - v0)) / area;
        let gamma = n.dot(edge1.cross(q - v0)) / area;

        if alpha >= 0 && beta >= 0 && gamma >= 0 {
            ray.tMax = tHit;
            return true;
        }
        else {
            return false;
        }
    }

    pub fn getAABB(&self) -> AABB{
        let v0 = self.vertex[self.v[0]];
        let v1 = self.vertex[self.v[1]];
        let v2 = self.vertex[self.v[2]];

        let min = v0.min(v1).min(v2);
        let max = v0.max(v1).max(v2);

        AABB::new(Vec3::splat(min), Vec3::splat(max));
    }
}

impl<'a> Primitive<'a> {
    pub fn intersect(&self, ray: &mut Ray) -> bool {
        match self {
            Primitive::Sphere(sphere) => sphere.intersect(ray),
            Primitive::Triangle(triangle) => triangle.intersect(ray),
        }
    }
    
    pub fn get_aabb(&self) -> AABB {
        match self {
            Primitive::Sphere(sphere) => sphere.get_aabb(),
            Primitive::Triangle(triangle) => triangle.get_aabb(),
        }
    }
}