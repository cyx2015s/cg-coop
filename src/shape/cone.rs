use crate::shape::mesh::{AsMesh, Mesh};
use std::f32::consts::PI;

pub struct Cone {
    pub radius: f32,  // 底面半径
    pub height: f32,  // 高度
    pub sectors: u16, // 切分份数
}

impl AsMesh for Cone {
    fn as_mesh(&self) -> Mesh {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        let sector_step = 2.0 * PI / self.sectors as f32;
        let half_h = self.height / 2.0;

        // 1. 侧面 
        
        let slant_height = (self.radius * self.radius + self.height * self.height).sqrt();
        let ny = self.radius / slant_height;
        let nz_base = self.height / slant_height;

        for i in 0..=self.sectors {
            let angle = i as f32 * sector_step;
            let x_sin = angle.sin();
            let z_cos = angle.cos();

            // 侧面法线
            let nx = x_sin * nz_base;
            let nz = z_cos * nz_base;

            // 锥顶
            vertices.push([0.0, half_h, 0.0]); // Top
            normals.push([nx, ny, nz]); // 法线指向上方侧面

            vertices.push([self.radius * x_sin, -half_h, self.radius * z_cos]); // Bottom
            normals.push([nx, ny, nz]);
        }

        // 侧面索引
        for i in 0..self.sectors {
            let top = i * 2;
            let bottom = top + 1;
            let next_top = top + 2;
            let next_bottom = bottom + 2;

            indices.push(top);
            indices.push(next_bottom);
            indices.push(bottom);
        }

        // 2. 底盖
        let offset = vertices.len() as u16;
        vertices.push([0.0, -half_h, 0.0]); // 中心
        normals.push([0.0, -1.0, 0.0]);
        let center_idx = offset;

        for i in 0..=self.sectors {
            let angle = i as f32 * sector_step;
            vertices.push([self.radius * angle.sin(), -half_h, self.radius * angle.cos()]);
            normals.push([0.0, -1.0, 0.0]);
        }

        for i in 0..self.sectors {
            indices.push(center_idx);
            indices.push(center_idx + 2 + i);
            indices.push(center_idx + 1 + i);
        }

        Mesh { vertices, normals, indices }
    }
}