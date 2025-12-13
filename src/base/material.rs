// use glium::implement_uniform_block;

pub struct Lambertian {
    pub ka: [f32; 3],
    pub kd: [f32; 3],
}

impl Lambertian {
    pub fn new(ka: [f32; 3], kd: [f32; 3]) -> Self {
        Self { ka, kd }
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
