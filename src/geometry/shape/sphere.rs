use imgui::Drag;

use crate::core::math::aabb::AABB;
use crate::geometry::shape::mesh;
use crate::geometry::shape::mesh::AsMesh;
use crate::scene::world::EditableMesh;
pub struct Sphere {
    pub radius: f32,
    pub col_divisions: u16,
    pub row_divisions: u16,
}

impl AsMesh for Sphere {
    fn as_mesh(&self) -> mesh::Mesh {
        let aabb = AABB::new_from_array([-self.radius; 3], [self.radius; 3]);
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();

        for row in 0..=self.row_divisions {
            let v = row as f32 / self.row_divisions as f32; // V 坐标 (0~1)
            let theta = std::f32::consts::PI * v;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for col in 0..=self.col_divisions {
                let u = col as f32 / self.col_divisions as f32; // U 坐标 (0~1)
                let phi = 2.0 * std::f32::consts::PI * u;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = self.radius * sin_theta * cos_phi;
                let y = self.radius * cos_theta;
                let z = self.radius * sin_theta * sin_phi;

                vertices.push([x, y, z]);
                normals.push([x / self.radius, y / self.radius, z / self.radius]);
                tex_coords.push([u, 1.0 - v]);
            }
        }

        for row in 0..self.row_divisions {
            for col in 0..self.col_divisions {
                let first = row * (self.col_divisions + 1) + col;
                let second = first + self.col_divisions + 1;

                indices.push(first);
                indices.push(second);
                indices.push(first + 1);

                indices.push(second);
                indices.push(second + 1);
                indices.push(first + 1);
            }
        }

        mesh::Mesh {
            vertices,
            normals,
            tex_coords,
            indices,
            aabb,
        }
    }
}

impl EditableMesh for Sphere {
    fn ui(&mut self, ui: &imgui::Ui) -> bool {
        let mut changed = false;
        ui.text("球体参数");
        changed |= Drag::new("半径").speed(0.1).build(ui, &mut self.radius);
        changed |= Drag::new("列划分数")
            .speed(1.0)
            .build(ui, &mut self.col_divisions);
        changed |= Drag::new("行划分数")
            .speed(1.0)
            .build(ui, &mut self.row_divisions);
        changed
    }
}
