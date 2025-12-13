use crate::implement_uniform_block_new;
// use glium::implement_uniform_block;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct MaterialBlock { 
    pub material: Material,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct Material { 
    pub ka: [f32; 3],
    pub _pad1: f32,
    pub kd: [f32; 3],
    pub _pad2: f32,
    pub ks: [f32; 3],
    pub ns: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            ka: [0.1, 0.1, 0.1],
            _pad1: 0.0,
            kd: [0.8, 0.8, 0.8],
            _pad2: 0.0,
            ks: [0.5, 0.5, 0.5],
            ns: 32.0,
        }
    }
}

implement_uniform_block_new!(Material, ka, _pad1, kd, _pad2, ks, ns);
implement_uniform_block_new!(MaterialBlock, material);

pub struct Phong {
    pub ka: [f32; 3],
    pub kd: [f32; 3],
    pub ks: [f32; 3],
    pub ns: f32,
}

impl Phong {
    pub fn new(ka: [f32; 3], kd: [f32; 3], ks: [f32; 3], ns: f32) -> Self {
        Self { ka, kd, ks, ns }
    }

    pub fn to_Material(&self) -> Material { 
        Material {
            ka: self.ka,
            _pad1: 0.0,
            kd: self.kd,
            _pad2: 0.0,
            ks: self.ks,
            ns: self.ns,
        }
    }

    // mat4
    // ka.x, kd.x, ks.x, 0
    // ka.y, kd.y, ks.y, 0
    // ka.z, kd.z, ks.z, 0
    // 0   , 0   , 0   , ns
    pub fn get_mat4_data(&self) -> [[f32; 4]; 4] {
        let mut data = [[0.0; 4]; 4];
        data[0][0] = self.ka[0];
        data[0][1] = self.ka[1];
        data[0][2] = self.ka[2];
        data[1][0] = self.kd[0];
        data[1][1] = self.kd[1];
        data[1][2] = self.kd[2];
        data[2][0] = self.ks[0];
        data[2][1] = self.ks[1];
        data[2][2] = self.ks[2];
        data[3][3] = self.ns;
        data
     }
}

pub struct Lambertian {
    pub ka: [f32; 3],
    pub kd: [f32; 3],
}

impl Lambertian {
    pub fn new(ka: [f32; 3], kd: [f32; 3]) -> Self {
        Self { ka, kd }
    }

    pub fn to_Material(&self) -> Material { 
        Material {
            ka: self.ka,
            _pad1: 0.0,
            kd: self.kd,
            _pad2: 0.0,
            ks: [0.0; 3],
            ns: 0.0,
        }
    }

    pub fn get_mat3_data(&self) -> [[f32; 3]; 3] {
        let mut data = [[0.0; 3]; 3];
        data[0][0] = self.ka[0];
        data[0][1] = self.ka[1];
        data[0][2] = self.ka[2];
        data[1][0] = self.kd[0];
        data[1][1] = self.kd[1];
        data[1][2] = self.kd[2];
        data
    }
}

// #[repr(C, align(16))]
// #[derive(Copy, Clone)]
// pub struct MaterialBlock {
//     pub material: Material,
// }

// #[repr(C, align(16))]
// #[derive(Copy, Clone, Debug)]
// pub struct Material {
//     pub ka: [f32; 3],
//     pub padding: f32,

//     pub kd: [f32; 3],
//     pub shininess: f32,
// }

// impl Material {
//     pub fn new() -> Self {
//         Self {
//             ka: [0.1, 0.1, 0.1],
//             padding: 0.0,
//             kd: [0.8, 0.8, 0.8],
//             shininess: 32.0,
//         }
//     }
// }

// implement_uniform_block!(Material, ka, padding, kd, shininess);
// implement_uniform_block!(MaterialBlock, material);
