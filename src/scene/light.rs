use crate::implement_uniform_block_new;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct LightBlock {
    pub lights: [Light; 32],
    pub num_lights: i32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct Light {
    pub color: [f32; 3],
    pub intensity: f32,

    pub position: [f32; 3],
    pub angle: f32,

    pub direction: [f32; 3],
    pub range: f32,

    pub kfactor: [f32; 3],
    pub light_type: i32,
}

implement_uniform_block_new!(
    Light, color, intensity, position, angle, direction, range, kfactor, light_type
);
implement_uniform_block_new!(LightBlock, lights, num_lights);

impl Light {
    pub fn new() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            position: [0.0, 0.0, 0.0],
            angle: 30.0,
            direction: [0.0, 0.0, -1.0],
            range: 10.0,
            kfactor: [0.0; 3],
            light_type: 0,
        }
    }

    pub fn get_light_space_matrix(&self) -> [[f32; 4];4] {
        let light_dir = glam::Vec3::from(self.direction).normalize();
        let light_pos = glam::Vec3::ZERO - light_dir * 20.0;
        let light_projection = glam::Mat4::orthographic_rh(-20.0, 20.0, -20.0, 20.0, 1.0, 50.0);
        let light_view = glam::Mat4::look_at_rh(light_pos, glam::Vec3::ZERO, glam::Vec3::Y);
        let light_space_matrix = light_projection * light_view;
        light_space_matrix.to_cols_array_2d()
    }

    pub fn is_directional(&self) -> bool {
        self.light_type == 1
    }

    pub fn is_point(&self) -> bool {
        self.light_type == 2
    }

    pub fn is_spot(&self) -> bool {
        self.light_type == 3
    }

    pub fn is_ambient(&self) -> bool {
        self.light_type == 0
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::new()
    }
}



// using mat4 to pass the light
// px, dx, cr, kc
// py, dy, cg, kl
// pz, dz, cb, kq
// intensity, angle, pad, light_type

pub struct AmbientLight {
    pub intensity: f32,
    pub color: [f32; 3],
}

#[derive(Copy, Clone)]
pub struct DirectionalLight {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
}

pub struct PointLight {
    pub position: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub kc: f32,
    pub kl: f32,
    pub kq: f32,
}

pub struct SpotLight {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub angle: f32,
    pub kc: f32,
    pub kl: f32,
    pub kq: f32,
}

impl AmbientLight {
    pub fn new(intensity: f32) -> Self {
        Self {
            intensity,
            color: [1.0, 1.0, 1.0],
        }
    }

    pub fn to_light(&self) -> Light {
        Light {
            color: self.color,
            intensity: self.intensity,
            position: [0.0, 0.0, 0.0],
            angle: 0.0,
            direction: [0.0, 0.0, -1.0],
            range: 0.0,
            kfactor: [0.0, 0.0, 0.0],
            light_type: 0,
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
    pub fn new(position: [f32; 3], direction: [f32; 3], intensity: f32, color: [f32; 3]) -> Self {
        Self {
            position,
            direction,
            intensity,
            color,
        }
    }

    pub fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, -1.0],
            intensity: 1.0,
            color: [1.0, 1.0, 1.0],
        }
    }

    pub fn get_forward(&self) -> [f32; 3] {
        self.direction
    }

    pub fn to_light(&self) -> Light {
        Light {
            color: self.color,
            intensity: self.intensity,
            position: self.position,
            angle: 0.0,
            direction: self.direction,
            range: 0.0,
            kfactor: [0.0, 0.0, 0.0],
            light_type: 1,
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
        data[2][0] = self.color[0];
        data[2][1] = self.color[1];
        data[2][2] = self.color[2];
        data[3][3] = 1.0;
        data
    }
}

impl PointLight {
    pub fn new(
        position: [f32; 3],
        intensity: f32,
        color: [f32; 3],
        kc: f32,
        kl: f32,
        kq: f32,
    ) -> Self {
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
            position: [0.0, 0.0, 0.0],
            intensity: 1.0,
            color: [1.0, 1.0, 1.0],
            kc: 1.0,
            kl: 0.0,
            kq: 1.0,
        }
    }

    pub fn to_light(&self) -> Light {
        Light {
            color: self.color,
            intensity: self.intensity,
            position: self.position,
            angle: 0.0,
            direction: [0.0, 0.0, -1.0],
            range: 0.0,
            kfactor: [self.kc, self.kl, self.kq],
            light_type: 2,
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
    pub fn new(
        position: [f32; 3],
        direction: [f32; 3],
        intensity: f32,
        color: [f32; 3],
        angle: f32,
        kc: f32,
        kl: f32,
        kq: f32,
    ) -> Self {
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
            position: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, -1.0],
            intensity: 1.0,
            color: [1.0, 1.0, 1.0],
            angle: 30.0,
            kc: 1.0,
            kl: 0.0,
            kq: 1.0,
        }
    }

    pub fn to_light(&self) -> Light {
        Light {
            color: self.color,
            intensity: self.intensity,
            position: self.position,
            angle: self.angle,
            direction: self.direction,
            range: 0.0,
            kfactor: [self.kc, self.kl, self.kq],
            light_type: 3,
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
