use imgui::Drag;

use crate::geometry::shape::mesh::{AsMesh, Mesh};
use crate::core::math::aabb::AABB;
use crate::scene::world::EditableMesh;
pub struct Cube {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

impl AsMesh for Cube {
    fn as_mesh(&self) -> Mesh {
        let w = self.width / 2.0;
        let h = self.height / 2.0;
        let d = self.depth / 2.0;

        let aabb = AABB::new_from_array([-w, -h, -d], [w, h, d]);
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();

        // Front Face (Z+)
        vertices.push([-w, -h, d]);
        normals.push([0.0, 0.0, 1.0]);
        tex_coords.push([0.0, 0.0]);
        vertices.push([w, -h, d]);
        normals.push([0.0, 0.0, 1.0]);
        tex_coords.push([1.0, 0.0]);
        vertices.push([w, h, d]);
        normals.push([0.0, 0.0, 1.0]);
        tex_coords.push([1.0, 1.0]);
        vertices.push([-w, h, d]);
        normals.push([0.0, 0.0, 1.0]);
        tex_coords.push([0.0, 1.0]);

        // Back Face (Z-)
        vertices.push([w, -h, -d]);
        normals.push([0.0, 0.0, -1.0]);
        tex_coords.push([0.0, 0.0]);
        vertices.push([-w, -h, -d]);
        normals.push([0.0, 0.0, -1.0]);
        tex_coords.push([1.0, 0.0]);
        vertices.push([-w, h, -d]);
        normals.push([0.0, 0.0, -1.0]);
        tex_coords.push([1.0, 1.0]);
        vertices.push([w, h, -d]);
        normals.push([0.0, 0.0, -1.0]);
        tex_coords.push([0.0, 1.0]);

        // Top Face (Y+)
        vertices.push([-w, h, d]);
        normals.push([0.0, 1.0, 0.0]);
        tex_coords.push([0.0, 0.0]);
        vertices.push([w, h, d]);
        normals.push([0.0, 1.0, 0.0]);
        tex_coords.push([1.0, 0.0]);
        vertices.push([w, h, -d]);
        normals.push([0.0, 1.0, 0.0]);
        tex_coords.push([1.0, 1.0]);
        vertices.push([-w, h, -d]);
        normals.push([0.0, 1.0, 0.0]);
        tex_coords.push([0.0, 1.0]);

        // Bottom Face (Y-)
        vertices.push([-w, -h, -d]);
        normals.push([0.0, -1.0, 0.0]);
        tex_coords.push([0.0, 0.0]);
        vertices.push([w, -h, -d]);
        normals.push([0.0, -1.0, 0.0]);
        tex_coords.push([1.0, 0.0]);
        vertices.push([w, -h, d]);
        normals.push([0.0, -1.0, 0.0]);
        tex_coords.push([1.0, 1.0]);
        vertices.push([-w, -h, d]);
        normals.push([0.0, -1.0, 0.0]);
        tex_coords.push([0.0, 1.0]);

        // Right Face (X+)
        vertices.push([w, -h, d]);
        normals.push([1.0, 0.0, 0.0]);
        tex_coords.push([0.0, 0.0]);
        vertices.push([w, -h, -d]);
        normals.push([1.0, 0.0, 0.0]);
        tex_coords.push([1.0, 0.0]);
        vertices.push([w, h, -d]);
        normals.push([1.0, 0.0, 0.0]);
        tex_coords.push([1.0, 1.0]);
        vertices.push([w, h, d]);
        normals.push([1.0, 0.0, 0.0]);
        tex_coords.push([0.0, 1.0]);

        // Left Face (X-)
        vertices.push([-w, -h, -d]);
        normals.push([-1.0, 0.0, 0.0]);
        tex_coords.push([0.0, 0.0]);
        vertices.push([-w, -h, d]);
        normals.push([-1.0, 0.0, 0.0]);
        tex_coords.push([1.0, 0.0]);
        vertices.push([-w, h, d]);
        normals.push([-1.0, 0.0, 0.0]);
        tex_coords.push([1.0, 1.0]);
        vertices.push([-w, h, -d]);
        normals.push([-1.0, 0.0, 0.0]);
        tex_coords.push([0.0, 1.0]);

        // Indices 生成
        for face in 0..6 {
            let start = face * 4;
            // Triangle 1
            indices.push(start);
            indices.push(start + 1);
            indices.push(start + 2);
            // Triangle 2
            indices.push(start);
            indices.push(start + 2);
            indices.push(start + 3);
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

impl EditableMesh for Cube {
    fn ui(&mut self, ui: &imgui::Ui) -> bool {
        let mut changed = false;
        ui.text("立方体参数");
        changed |= Drag::new("宽度").speed(0.1).build(ui, &mut self.width);
        changed |= Drag::new("高度").speed(0.1).build(ui, &mut self.height);
        changed |= Drag::new("深度").speed(0.1).build(ui, &mut self.depth);
        changed
    }
}