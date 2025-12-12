
// use glium::implement_uniform_block;
// use glium::program::{ BlockLayout};
// use glium::uniforms::{ LayoutMismatchError, UniformBlock, UniformType };

// using mat4 to pass the light
// px, dx, cr, kc
// py, dy, cg, kl
// pz, dz, cb, kq
// intensity, angle, pad, light_type 

pub struct AmbientLight {
    pub intensity : f32,
    pub color : [f32; 3],
}

pub struct DirectionalLight {
    pub position : [f32; 3],
    pub direction : [f32; 3],
    pub intensity : f32,
    pub color : [f32; 3],
}

pub struct PointLight {
    pub position : [f32; 3],
    pub intensity : f32,
    pub color : [f32; 3],
    pub kc : f32,
    pub kl : f32,
    pub kq : f32,
}

pub struct SpotLight {
    pub position : [f32; 3],
    pub direction : [f32; 3],
    pub intensity : f32,
    pub color : [f32; 3],
    pub angle : f32,
    pub kc : f32,
    pub kl : f32,
    pub kq : f32,
}

impl AmbientLight {
    pub fn new(intensity : f32) -> Self {
        Self {
            intensity,
            color : [1.0, 1.0, 1.0],
        }
    }

    pub fn get_mat4_data(&self) -> [[f32; 4]; 4] {
        let mut data = [[0.0; 4]; 4];
        data[0][3] = self.intensity;
        data[2][0] = self.color[0];
        data[2][1] = self.color[1];
        data[2][2] = self.color[2];
        data[3][3] = 0.0;
        data
    }
}

impl DirectionalLight {
    pub fn new(position: [f32; 3], direction: [f32; 3] , intensity : f32, color : [f32; 3]) -> Self {
        Self {
            position,
            direction,
            intensity,
            color,
        }
    }

    pub fn default() -> Self {
        Self {
            position : [0.0, 0.0, 0.0],
            direction : [0.0, 0.0, -1.0],
            intensity : 1.0,
            color : [1.0, 1.0, 1.0],
        }
    }

    pub fn get_forward(&self) -> [f32; 3] {
        self.direction
    }

    pub fn get_mat4_data(&self) -> [[f32; 4]; 4] {
        let mut data = [[0.0; 4]; 4];
        data[0][0] = self.position[0];
        data[0][1] = self.position[1];
        data[0][2] = self.position[2];
        data[0][3] = self.intensity;
        data[1][0] = self.direction[0];
        data[1][1] = self.direction[1];
        data[1][2] = self.direction[2];
        data[2][0] = self.color[0];
        data[2][1] = self.color[1];
        data[2][2] = self.color[2];
        data[3][3] = 1.0;
        data
    }


}

impl PointLight {
    pub fn new(position: [f32; 3], intensity : f32, color : [f32; 3], kc : f32, kl : f32, kq : f32) -> Self {
        Self {
            position,
            intensity,
            color,
            kc,
            kl,
            kq,
        }
    }

    pub fn default() -> Self {
        Self {
            position : [0.0, 0.0, 0.0],
            intensity : 1.0,
            color : [1.0, 1.0, 1.0],
            kc : 1.0,
            kl : 0.0,
            kq : 1.0,
        }
    }

    pub fn get_mat4_data(&self) -> [[f32; 4]; 4] {
        let mut data = [[0.0; 4]; 4];
        data[0][0] = self.position[0];
        data[0][1] = self.position[1];
        data[0][2] = self.position[2];
        data[0][3] = self.intensity;
        data[2][0] = self.color[0];
        data[2][1] = self.color[1];
        data[2][2] = self.color[2];
        data[3][0] = self.kc;
        data[3][1] = self.kl;
        data[3][2] = self.kq;
        data[3][3] = 2.0;
        data
    }

}

impl SpotLight {
    pub fn new(position: [f32; 3],direction: [f32; 3], intensity : f32, color : [f32; 3], angle : f32, kc : f32, kl : f32, kq : f32) -> Self {
        Self {
            position,
            direction,
            intensity,
            color,
            angle,
            kc,
            kl,
            kq,
        }
    }

    pub fn default() -> Self {
        Self {
            position : [0.0, 0.0, 0.0],
            direction : [0.0, 0.0, -1.0],
            intensity : 1.0,
            color : [1.0, 1.0, 1.0],
            angle : 30.0,
            kc : 1.0,
            kl : 0.0,
            kq : 1.0,
        }
    }

    pub fn get_mat4_data(&self) -> [[f32; 4]; 4] {
        let mut data = [[0.0; 4]; 4];
        data[0][0] = self.position[0];
        data[0][1] = self.position[1];
        data[0][2] = self.position[2];
        data[0][3] = self.intensity;
        data[1][0] = self.direction[0];
        data[1][1] = self.direction[1];
        data[1][2] = self.direction[2];
        data[1][3] = self.angle;
        data[2][0] = self.color[0];
        data[2][1] = self.color[1];
        data[2][2] = self.color[2];
        data[3][0] = self.kc;
        data[3][1] = self.kl;
        data[3][2] = self.kq;
        data[3][3] = 3.0;
        data
    }
}

// #[repr(C, align(16))]
// #[derive(Copy, Clone)]
// pub struct LightBlock {
//     pub lights: [Light; 32],
//     pub lightCount: i32,
//     pub _pad: [i32; 3],
// }

// #[derive(Copy, Clone)]
// struct Vertex {
//     value: [f32; 4],
// }

// #[repr(C, align(16))]
// #[derive(Copy, Clone, Debug)]
// pub struct Light {
//     pub color: [f32; 3],
//     pub intensity: f32,
    
//     pub position: [f32; 3],
//     pub angle: f32,

//     pub direction: [f32; 3],
//     pub range: f32,

//     pub kfactor: [f32; 3],
//     pub light_type: i32,

// }

// impl Light {
//     pub fn new() -> Self {
//         Self {
//             color: [1.0, 1.0, 1.0],
//             intensity: 1.0,
//             position: [0.0, 0.0, 0.0],
//             angle: 30.0,
//             direction: [0.0, 0.0, -1.0],
//             range: 10.0,
//             light_type: 0,
//             kfactor: [1.0, 0.0, 1.0],
//         }
//     }
// }

// implement_uniform_block!(Light, color, intensity, position, angle, direction, range, kfactor, light_type);
// implement_uniform_block!(LightBlock, lights, lightCount, _pad);

// impl UniformBlock for Vertex {
//     fn matches(layout: &BlockLayout, base_offset: usize) -> Result<(), LayoutMismatchError> {
//         if let BlockLayout::Struct { members } = layout {
//             if members.len() != 1 {
//                 return LayoutMismatchError::;
//             }

//             // 检查 position
//             if members[0].0 != "value" {
//                 return Err(LayoutMismatchError);
//             }
            
//             if let BlockLayout::BasicType { ty, offset_in_buffer} = members[0].1 {
//                 if *ty != UniformType::FloatVec4 {
//                     return Err(LayoutMismatchError);
//                 } else if *offset_in_buffer != base_offset {
//                     return Err(LayoutMismatchError);
//                 }
//             } else {
//                 Err(LayoutMismatchError)
//             }
            
            

//             Ok(())
//         } else {
//             Err(LayoutMismatchError)
//         }
//     }

//     fn build_layout(_base_offset: usize) -> BlockLayout {
//         BlockLayout::Struct {
//             members: vec![
//                 ("value".to_string(), BlockLayout::BasicType {
//                     ty: UniformType::FloatVec4,
//                     offset_in_buffer: 0,
//                 }),
//             ],
//         }
//     }
// }