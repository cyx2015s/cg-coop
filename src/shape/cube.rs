use crate::shape::mesh::{AsMesh, Mesh};

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

        // 定义 24 个顶点 (6面 * 4点)
        // 每个面一组，顺序为：左下，右下，右上，左上 
        let raw_vertices = vec![
            // Front face (z = +d)
            [-w, -h,  d], [ w, -h,  d], [ w,  h,  d], [-w,  h,  d],
            // Back face (z = -d)
            [ w, -h, -d], [-w, -h, -d], [-w,  h, -d], [ w,  h, -d],
            // Top face (y = +h)
            [-w,  h,  d], [ w,  h,  d], [ w,  h, -d], [-w,  h, -d],
            // Bottom face (y = -h)
            [-w, -h, -d], [ w, -h, -d], [ w, -h,  d], [-w, -h,  d],
            // Right face (x = +w)
            [ w, -h,  d], [ w, -h, -d], [ w,  h, -d], [ w,  h,  d],
            // Left face (x = -w)
            [-w, -h, -d], [-w, -h,  d], [-w,  h,  d], [-w,  h, -d],
        ];

        // 对应的法线 (24 个)
        let raw_normals = vec![
            // Front (0, 0, 1)
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
            // Back (0, 0, -1)
            [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
            // Top (0, 1, 0)
            [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            // Bottom (0, -1, 0)
            [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
            // Right (1, 0, 0)
            [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            // Left (-1, 0, 0)
            [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
        ];

        // 索引
        let mut indices: Vec<u16> = Vec::new();
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
            vertices: raw_vertices,
            normals: raw_normals,
            indices,
        }
    }
}