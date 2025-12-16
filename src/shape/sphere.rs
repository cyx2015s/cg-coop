use crate::shape::mesh::AsMesh;

pub struct Sphere {
    pub radius: f32,
    pub col_divisions: u16,
    pub row_divisions: u16,
}

impl AsMesh for Sphere {
    fn as_mesh(&self) -> crate::shape::mesh::Mesh {
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
                indices.push((first + 1));

                indices.push(second);
                indices.push((second + 1));
                indices.push((first + 1));
            }
        }

        crate::shape::mesh::Mesh {
            vertices,
            normals,
            tex_coords,
            indices,
        }
    }
}
