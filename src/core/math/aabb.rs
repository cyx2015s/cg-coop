use std::ops::{ Index };
use glam::f32::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct AABB{
    pub min: Vec3,
    pub max: Vec3
}

impl AABB{

    pub fn default() -> Self{
        Self{
            min: Vec3::INFINITY,
            max: Vec3::NEG_INFINITY,
        }
    }
    pub fn new(min: Vec3, max: Vec3) -> Self{
        Self{
            min,
            max
        }
    }

    pub fn new_from_array(min: [f32; 3], max: [f32; 3]) -> Self {
        Self::new(Vec3::from_array(min), Vec3::from_array(max))
    }

    // pub fn intersect(&self, ray: &Ray) -> bool{
    //     let invDir = Vec3::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);
    //     let sign = Vec3::new(if ray.d.x < 0.0 { 1 } else { 0 }, if ray.d.y < 0.0 { 1 } else { 0 }, if ray.d.z < 0.0 { 1 } else { 0 });
    //     return self.intersect_full(ray, invDir, sign);
    // }

    // pub fn intersect_full(&self, ray: &Ray, invDir: Vec3, sign: Vec3) -> bool{ 
    //     let aabb = self;
    //     let o = &ray.o;
    //     let d = &ray.d;

    //     f32 tMinX = (self[sign.x].x - o.x) * invDir.x;
    //     f32 tMaxX = (self[1 - sign.x].x - o.x) * invDir.x;
    //     f32 tMinY = (self[sign.y].y - o.y) * invDir.y;
    //     f32 tMaxY = (self[1 - sign.y].y - o.y) * invDir.y;
    //     f32 tMinZ = (self[sign.z].z - o.z) * invDir.z;
    //     f32 tMaxZ = (self[1 - sign.z].z - o.z) * invDir.z;

    //     let tMin = tMinX.max(tMinY).max(tMinZ);
    //     let tMax = tMaxX.min(tMaxY).min(tMaxZ);

    //     return tMin < tMax && tMax > 0.0 && tMin < ray.tMax;
    // }
    pub fn get_half_extents(&self) -> Vec3 { 
        (self.max - self.min) * 0.5
    }

    pub fn get_global_aabb(&self, model_matrix:glam::f32::Mat4) -> AABB { 
        let center = (self.min + self.max) * 0.5;
        let global_center = (model_matrix * glam::f32::Vec4::from((center,1.0))).truncate();
        let extents = self.max - center;

        let right   = model_matrix.x_axis.truncate() * extents.x;
        let up      = model_matrix.y_axis.truncate() * extents.y;
        let forward = model_matrix.z_axis.truncate() * extents.z;

        let new_i = Vec3::X.dot(right).abs()
            + Vec3::X.dot(up).abs()
            + Vec3::X.dot(forward).abs();

        let new_j = Vec3::Y.dot(right).abs()
                    + Vec3::Y.dot(up).abs()
                    + Vec3::Y.dot(forward).abs();

        let new_k = Vec3::Z.dot(right).abs()
                    + Vec3::Z.dot(up).abs()
                    + Vec3::Z.dot(forward).abs();
        let global_aabb = AABB::new_from_array(
            [global_center.x - new_i, global_center.y - new_j, global_center.z - new_k], 
            [global_center.x + new_i, global_center.y + new_j, global_center.z + new_k]
        );
        
        return global_aabb;
    }
    pub fn union_point_array(&mut self, v: [f32; 3]) {
        self.min = self.min.min(glam::f32::Vec3::from_array(v));
        self.max = self.max.max(glam::f32::Vec3::from_array(v));
    }

    pub fn union_aabb(&mut self, b: &AABB) {
         self.min = self.min.min(b.min);
         self.max = self.max.max(b.max);
    }
}

impl Index<usize> for AABB {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.min,
            1 => &self.max,
            _ => panic!("Index out of bounds: {}", index),
        }
    }
}



pub fn union_aabb_inplace(dest: &mut AABB, src: &AABB) {
    dest.min = dest.min.min(src.min);
    dest.max = dest.max.max(src.max);
}

pub fn union_aabb_point(a: &AABB, p: Vec3) -> AABB {
    AABB {
        min: Vec3::new(a.min.x.min(p.x), a.min.y.min(p.y), a.min.z.min(p.z)),
        max: Vec3::new(a.max.x.max(p.x), a.max.y.max(p.y), a.max.z.max(p.z))
    }
}



pub fn union_aabb_point_inplace(dest: &mut AABB, p: Vec3) {
    dest.min = dest.min.min(p);
    dest.max = dest.max.max(p);
}

pub fn maximum_dim(a: &AABB) -> usize {
    let diag = a.max - a.min;
    if diag.x > diag.y && diag.x > diag.z {
        0
    } else if diag.y > diag.z {
        1
    } else {
        2
    }
}