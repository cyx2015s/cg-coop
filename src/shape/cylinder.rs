use crate::shape::mesh::{AsMesh, Mesh};
use std::f32::consts::PI;

pub struct Cylinder {
    pub bottom_radius: f32,
    pub top_radius: f32,
    pub height: f32,
    pub sectors: u16,
}

impl AsMesh for Cylinder {
    fn as_mesh(&self) -> Mesh {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();

        let sector_step = 2.0 * PI / self.sectors as f32;
        let half_h = self.height / 2.0;
        let radius_diff = self.bottom_radius - self.top_radius;
        let slant_height = (radius_diff * radius_diff + self.height * self.height).sqrt();
        let nxz = self.height / slant_height;
        let ny = radius_diff / slant_height;

        // 1. 侧面
        for i in 0..=self.sectors {
            let u = i as f32 / self.sectors as f32; // U坐标环绕一圈
            let angle = i as f32 * sector_step;
            let x_sin = angle.sin();
            let z_cos = angle.cos();

            let nx = x_sin * nxz;
            let nz = z_cos * nxz;

            // 顶圈
            vertices.push([self.top_radius * x_sin, half_h, self.top_radius * z_cos]);
            normals.push([nx, ny, nz]);
            tex_coords.push([u, 1.0]); // 顶部 V=1

            // 底圈
            vertices.push([self.bottom_radius * x_sin, -half_h, self.bottom_radius * z_cos]);
            normals.push([nx, ny, nz]);
            tex_coords.push([u, 0.0]); // 底部 V=0
        }

        for i in 0..self.sectors {
            let top1 = i * 2;
            let bottom1 = top1 + 1;
            let top2 = top1 + 2;
            let bottom2 = bottom1 + 2;
            indices.push(bottom1); indices.push(top1); indices.push(top2);
            indices.push(bottom1); indices.push(top2); indices.push(bottom2);
        }

        // 2. 顶盖
        let offset = vertices.len() as u16;
        vertices.push([0.0, half_h, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        tex_coords.push([0.5, 0.5]); // 中心
        let top_center_idx = offset;

        for i in 0..=self.sectors {
            let angle = i as f32 * sector_step;
            let x = angle.sin();
            let z = angle.cos();
            vertices.push([self.top_radius * x, half_h, self.top_radius * z]);
            normals.push([0.0, 1.0, 0.0]);
            // 映射到 0~1 的平面
            tex_coords.push([(x + 1.0) * 0.5, (z + 1.0) * 0.5]);
        }
        for i in 0..self.sectors {
            indices.push(top_center_idx);
            indices.push(top_center_idx + 1 + i);
            indices.push(top_center_idx + 2 + i);
        }

        // 3. 底盖
        let offset = vertices.len() as u16;
        vertices.push([0.0, -half_h, 0.0]);
        normals.push([0.0, -1.0, 0.0]);
        tex_coords.push([0.5, 0.5]);
        let bottom_center_idx = offset;

        for i in 0..=self.sectors {
            let angle = i as f32 * sector_step;
            let x = angle.sin();
            let z = angle.cos();
            vertices.push([self.bottom_radius * x, -half_h, self.bottom_radius * z]);
            normals.push([0.0, -1.0, 0.0]);
            tex_coords.push([(x + 1.0) * 0.5, (z + 1.0) * 0.5]);
        }
        for i in 0..self.sectors {
            indices.push(bottom_center_idx);
            indices.push(bottom_center_idx + 2 + i);
            indices.push(bottom_center_idx + 1 + i);
        }

        Mesh { vertices, normals, tex_coords, indices }
    }
}