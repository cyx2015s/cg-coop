use glam::f32::Vec3;
pub struct Ray{
    pub o : Vec3,
    pub d : Vec3,
    pub tMax : f32,
}

impl Ray{
    pub fn new(o:Vec3,d:Vec3)->Self{
        Self{
            o,
            d,
            tMax:f32::INFINITY,
        }
    }

    pub fn default()->Self{
        Self::new(Vec3::ZERO,Vec3::ZERO)
    }

}