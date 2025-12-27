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
    pub const NONE: Self = Self {
        color: [0.0, 0.0, 0.0],
        intensity: 0.0,
        position: [0.0, 0.0, 0.0],
        angle: 0.0,
        direction: [0.0, 0.0, 0.0],
        range: 0.0,
        kfactor: [0.0, 0.0, 0.0],
        light_type: 0,
    };

    pub const AMBIENT: Self = Self {
        color: [1.0, 1.0, 1.0],
        intensity: 0.2,
        position: [0.0, 0.0, 0.0],
        angle: 0.0,
        direction: [0.0, 0.0, 0.0],
        range: 0.0,
        kfactor: [0.0, 0.0, 0.0],
        light_type: 0,
    };

    pub const DERECTIONAL: Self = Self {
        color: [1.0, 1.0, 1.0],
        intensity: 1.0,
        position: [0.0, 0.0, 0.0],
        angle: 0.0,
        direction: [0.0, -1.0, 0.0],
        range: 0.0,
        kfactor: [0.0, 0.0, 0.0],
        light_type: 1,
    };

    pub const POINT: Self = Self {
        color: [1.0, 1.0, 1.0],
        intensity: 1.0,
        position: [0.0, 0.0, 0.0],
        angle: 0.0,
        direction: [0.0, -1.0, 0.0],
        range: 0.0,
        kfactor: [1.0, 0.09, 0.032],
        light_type: 2,
    };

    pub const SPOT: Self = Self {
        color: [1.0, 1.0, 1.0],
        intensity: 1.0,
        position: [0.0, 0.0, 0.0],
        angle: 12.5,
        direction: [0.0, -1.0, 0.0],
        range: 10.0,
        kfactor: [1.0, 0.09, 0.032],
        light_type: 3,
    };
    pub fn get_light_space_matrix(&self) -> [[f32; 4]; 4] {
        let light_dir = glam::Vec3::from(self.direction).normalize();
        let light_pos = glam::Vec3::ZERO - light_dir * 20.0;
        let light_projection = glam::Mat4::orthographic_rh_gl(-20.0, 20.0, -20.0, 20.0, 1.0, 50.0);
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
