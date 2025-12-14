use crate::shape::mesh::{AsMesh, Mesh};
use std::f32::consts::PI;

pub struct Cylinder {
    pub bottom_radius: f32, // 底面半径
    pub top_radius: f32,    // 顶面半径
    pub height: f32,        // 高度
    pub sectors: u16,       // 切分份数
}

impl AsMesh for Cylinder {
    fn as_mesh(&self) -> Mesh {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        let sector_step = 2.0 * PI / self.sectors as f32;
        let half_h = self.height / 2.0;

        // 1. 计算底面和顶面的半径差
        let radius_diff = self.bottom_radius - self.top_radius;
        // 2. 计算侧面的斜边长度 (勾股定理)
        let slant_height = (radius_diff * radius_diff + self.height * self.height).sqrt();
        // 3. 计算法线的分量
        // nxz: 水平方向的分量 (高度 / 斜边长)
        // ny:  垂直方向的分量 (半径差 / 斜边长)
        let nxz = self.height / slant_height;
        let ny = radius_diff / slant_height;

        // 1. 生成侧面
        for i in 0..=self.sectors {
            let angle = i as f32 * sector_step;
            let x_sin = angle.sin();
            let z_cos = angle.cos();

            // 使用计算好的倾斜法线
            let nx = x_sin * nxz;
            let nz = z_cos * nxz;

            // 顶圈顶点
            vertices.push([self.top_radius * x_sin, half_h, self.top_radius * z_cos]);
            normals.push([nx, ny, nz]);

            // 底圈顶点
            vertices.push([self.bottom_radius * x_sin, -half_h, self.bottom_radius * z_cos]);
            normals.push([nx, ny, nz]);
        }

        // 侧面索引构建
        for i in 0..self.sectors {
            let top1 = i * 2;
            let bottom1 = top1 + 1;
            let top2 = top1 + 2;
            let bottom2 = bottom1 + 2;

            // 三角形 1
            indices.push(bottom1);
            indices.push(top1);
            indices.push(top2);

            // 三角形 2
            indices.push(bottom1);
            indices.push(top2);
            indices.push(bottom2);
        }

        // 记录目前的顶点数偏移量，给盖子用
        let offset = vertices.len() as u16;

        // 2. 生成顶盖
        // 中心点
        vertices.push([0.0, half_h, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        let top_center_idx = offset;

        // 顶盖外圈
        for i in 0..=self.sectors {
            let angle = i as f32 * sector_step;
            vertices.push([self.top_radius * angle.sin(), half_h, self.top_radius * angle.cos()]);
            normals.push([0.0, 1.0, 0.0]);
        }
        
        // 顶盖索引
        for i in 0..self.sectors {
            indices.push(top_center_idx);
            indices.push(top_center_idx + 1 + i);
            indices.push(top_center_idx + 2 + i);
        }

        // 3. 生成底盖
        let offset = vertices.len() as u16;
        // 中心点
        vertices.push([0.0, -half_h, 0.0]);
        normals.push([0.0, -1.0, 0.0]); // 法线向下
        let bottom_center_idx = offset;

        // 底盖外圈
        for i in 0..=self.sectors {
            let angle = i as f32 * sector_step;
            vertices.push([self.bottom_radius * angle.sin(), -half_h, self.bottom_radius * angle.cos()]);
            normals.push([0.0, -1.0, 0.0]);
        }

        // 底盖索引 (保证面朝外)
        for i in 0..self.sectors {
            indices.push(bottom_center_idx);
            indices.push(bottom_center_idx + 2 + i);
            indices.push(bottom_center_idx + 1 + i);
        }

        Mesh { vertices, normals, indices }
    }
}