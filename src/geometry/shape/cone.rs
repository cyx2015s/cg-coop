use crate::geometry::shape::mesh::{AsMesh, Mesh};
use crate::core::math::aabb::AABB;
use crate::scene::world::EditableMesh;
use std::f32::consts::PI;

pub struct Cone {
    pub radius: f32,
    pub height: f32,
    pub sectors: u16,
}

impl AsMesh for Cone {
    fn as_mesh(&self) -> Mesh {
        let aabb = AABB::new_from_array([-self.radius, -self.height / 2.0, -self.radius], [self.radius, self.height / 2.0, self.radius]);
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();

        let sector_step = 2.0 * PI / self.sectors as f32;
        let half_h = self.height / 2.0;

        // 计算侧面法线
        // 斜边长度
        let slant_height = (self.radius * self.radius + self.height * self.height).sqrt();
        // 法线分量: 水平分量 (h / slant), 垂直分量 (r / slant)
        let nxz = self.height / slant_height;
        let ny = self.radius / slant_height;

        // 1. 生成侧面
        for i in 0..=self.sectors {
            let u = i as f32 / self.sectors as f32; // U 坐标
            let angle = i as f32 * sector_step;
            let x_sin = angle.sin();
            let z_cos = angle.cos();

            let nx = x_sin * nxz;
            let nz = z_cos * nxz;

            // 顶点
            // 多个重合的顶点以支持纹理接缝
            vertices.push([0.0, half_h, 0.0]);
            normals.push([nx, ny, nz]);
            tex_coords.push([u, 1.0]);

            // 底圈点
            vertices.push([self.radius * x_sin, -half_h, self.radius * z_cos]);
            normals.push([nx, ny, nz]);
            tex_coords.push([u, 0.0]); // V=0 在底部
        }

        // 索引构建
        for i in 0..self.sectors {
            let top = i * 2;
            let bottom = top + 1;
            let _next_top = top + 2;
            let next_bottom = bottom + 2;

            // 圆锥侧面只需要一个三角形 (Top -> Bottom -> Next_Bottom)
            indices.push(bottom);
            indices.push(top);
            indices.push(next_bottom);
        }

        // 2. 生成底盖
        let offset = vertices.len() as u16;
        // 中心点
        vertices.push([0.0, -half_h, 0.0]);
        normals.push([0.0, -1.0, 0.0]);
        tex_coords.push([0.5, 0.5]);
        let center_idx = offset;

        for i in 0..=self.sectors {
            let angle = i as f32 * sector_step;
            let x = angle.sin();
            let z = angle.cos();

            vertices.push([self.radius * x, -half_h, self.radius * z]);
            normals.push([0.0, -1.0, 0.0]);
            // 平面投影映射
            tex_coords.push([(x + 1.0) * 0.5, (z + 1.0) * 0.5]);
        }

        for i in 0..self.sectors {
            indices.push(center_idx);
            indices.push(center_idx + 2 + i);
            indices.push(center_idx + 1 + i);
        }

        Mesh {
            vertices,
            normals,
            tex_coords,
            indices,
            aabb,
        }
    }
}

impl EditableMesh for Cone {
    fn ui(&mut self, ui: &imgui::Ui) -> bool {
        let mut changed = false;
        ui.text("锥体参数");
        changed |= imgui::Drag::new("底半径").speed(0.05).build(ui, &mut self.radius);
        changed |= imgui::Drag::new("高度").speed(0.1).build(ui, &mut self.height);
        changed |= imgui::Drag::new("精度").speed(1.0).build(ui, &mut self.sectors);
        changed
    }
    
    fn debug_ui(&mut self, _ui: &imgui::Ui) {
    
    }
}