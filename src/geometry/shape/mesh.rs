use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::core::math::aabb::AABB;
use crate::scene::world::EditableMesh;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
    pub aabb: AABB,
}

pub trait AsMesh {
    fn as_mesh(&self) -> Mesh;
}

impl AsMesh for Mesh {
    fn as_mesh(&self) -> Mesh {
        self.clone()
    }
}

impl EditableMesh for Mesh {
    fn ui(&mut self, ui: &imgui::Ui) -> bool {
        ui.text("网格模型编辑能力受限。请在外部 3D 软件中修改后重新导入。");
        false
    }
}

impl Mesh {
    /// 导出为 OBJ 文件
    pub fn save_obj<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "# Exported by CG-Coop")?;

        for v in &self.vertices {
            writeln!(file, "v {} {} {}", v[0], v[1], v[2])?;
        }

        // 导出纹理坐标
        for vt in &self.tex_coords {
            writeln!(file, "vt {} {}", vt[0], vt[1])?;
        }

        for n in &self.normals {
            writeln!(file, "vn {} {} {}", n[0], n[1], n[2])?;
        }

        // 修改面数据的导出格式: v/vt/vn
        // OBJ索引从1开始
        for chunk in self.indices.chunks(3) {
            if chunk.len() == 3 {
                let i0 = chunk[0] + 1;
                let i1 = chunk[1] + 1;
                let i2 = chunk[2] + 1;
                // 格式：顶点索引/纹理索引/法线索引
                writeln!(
                    file,
                    "f {}/{}/{} {}/{}/{} {}/{}/{}",
                    i0, i0, i0, i1, i1, i1, i2, i2, i2
                )?;
            }
        }

        println!("Mesh saved successfully.");
        Ok(())
    }

    /// 从 OBJ 文件导入
    pub fn load_obj<P: AsRef<Path>>(path: P) -> Result<Mesh, String> {
        let mut aabb = AABB::default();
        let path = path.as_ref();

        let load_options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        };

        let (models, _materials) = tobj::load_obj(path, &load_options)
            .map_err(|e| format!("Failed to load OBJ: {}", e))?;

        if let Some(model) = models.first() {
            let mesh = &model.mesh;

            let vertices: Vec<[f32; 3]> = mesh
                .positions
                .chunks(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();
            for v in &vertices {
                aabb.union_point_array(*v);
            }
            // 读取纹理坐标
            let tex_coords: Vec<[f32; 2]> = if mesh.texcoords.is_empty() {
                println!("警告：OBJ 模型缺少纹理坐标 (UV)，正在自动生成球形映射 UV...");

                // 自动生成球形 UV 映射 (适用于茶壶、球体等)
                vertices
                    .iter()
                    .map(|v| {
                        let x = v[0];
                        let y = v[1];
                        let z = v[2];

                        // 计算到原点的距离
                        let len = (x * x + y * y + z * z).sqrt();

                        if len > 0.0001 {
                            // 使用 atan2 计算角度，映射到 0~1
                            let theta = z.atan2(x); // 经度
                            let phi = (y / len).asin(); // 纬度

                            let u = (theta + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
                            let v = (phi + std::f32::consts::PI / 2.0) / std::f32::consts::PI;

                            [u, v]
                        } else {
                            [0.0, 0.0]
                        }
                    })
                    .collect()
            } else {
                mesh.texcoords.chunks(2).map(|c| [c[0], c[1]]).collect()
            };

            // 智能法线计算
            let normals: Vec<[f32; 3]> = if mesh.normals.is_empty() {
                println!("OBJ 模型缺少法线，正在自动计算平滑法线...");
                compute_smooth_normals(&vertices, &mesh.indices)
            } else {
                mesh.normals.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
            };

            let indices: Vec<u16> = mesh.indices.iter().map(|&i| i as u16).collect();

            println!(
                "Loaded OBJ: {} vertices, {} indices",
                vertices.len(),
                indices.len()
            );

            Ok(Mesh {
                vertices,
                normals,
                tex_coords,
                indices,
                aabb,
            })
        } else {
            Err("OBJ file contains no models".to_string())
        }
    }

    pub fn get_indices(&self) -> &[u16] {
        &self.indices
    }

    pub fn get_vertices(&self) -> &[[f32; 3]]  {
        &self.vertices
    }

    pub fn get_tex_coords(&self) -> &[[f32; 2]] {
        &self.tex_coords
    }

    pub fn get_normals(&self) -> &[[f32; 3]] {
        &self.normals
    }

    pub fn get_vertex(&self, index: usize) -> &[f32; 3] {
        return &self.vertices[index];
    }
}

// 辅助函数：计算平滑法线
fn compute_smooth_normals(vertices: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0, 0.0, 0.0]; vertices.len()];

    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            let v0 = vertices[i0];
            let v1 = vertices[i1];
            let v2 = vertices[i2];

            let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

            let normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];

            for idx in [i0, i1, i2] {
                normals[idx][0] += normal[0];
                normals[idx][1] += normal[1];
                normals[idx][2] += normal[2];
            }
        }
    }

    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 0.0 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        } else {
            *n = [0.0, 1.0, 0.0];
        }
    }

    normals
}


fn compute_closest_point(
    vertices: &[[f32; 3]],
    origin: [f32; 3],
    direction: [f32; 3],
) -> Option<([f32; 3], f32)> {
    let mut closest_point: Option<[f32; 3]> = None;
    let mut max_cos = 0.0;
    let direction = glam::vec3(direction[0], direction[1], direction[2]);
    let direction = direction.normalize();
    for v in vertices {
        let to_vertex = glam::vec3(v[0] - origin[0], v[1] - origin[1], v[2] - origin[2]);
        let t = to_vertex.dot(direction);

        if t >= 0.0 {
            let cos_angle = t / to_vertex.length();
            if cos_angle > max_cos {
                max_cos = cos_angle;
                closest_point = Some(*v);
            }
        }
    }

    closest_point.map(|p| (p, max_cos))
}

#[test]
fn test_compute_closest_point() {
    let vertices = vec![[1.0, 0.0, 0.0], [1.0, 0.9, 0.0], [0.0, 0.0, 1.0]];
    let origin = [0.0, 0.0, 0.0];
    let direction = [1.0, 1.0, 0.0];

    let closest = compute_closest_point(&vertices, origin, direction);
    assert!(closest.is_some_and(|v| v.0 == [1.0, 0.9, 0.0]));
    println!("Closest point: {:?}", closest);
}

impl Mesh {
    /// 计算与给定方向最接近的顶点
    pub fn compute_closest_point(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
    ) -> Option<([f32; 3], f32)> {
        compute_closest_point(&self.vertices, origin, direction)
    }

    pub fn compute_intersecting_face(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
    ) -> Option<([u16; 3], [f32; 3])> {
        let direction = glam::vec3(direction[0], direction[1], direction[2]).normalize();
        let origin = glam::vec3(origin[0], origin[1], origin[2]);

        let mut intersected_face: Option<([u16; 3], [f32; 3])> = None;
        let mut min_t = f32::MAX;
        for chunk in self.indices.chunks(3) {
            if chunk.len() == 3 {
                let v0 = glam::vec3(
                    self.vertices[chunk[0] as usize][0],
                    self.vertices[chunk[0] as usize][1],
                    self.vertices[chunk[0] as usize][2],
                );
                let v1 = glam::vec3(
                    self.vertices[chunk[1] as usize][0],
                    self.vertices[chunk[1] as usize][1],
                    self.vertices[chunk[1] as usize][2],
                );
                let v2 = glam::vec3(
                    self.vertices[chunk[2] as usize][0],
                    self.vertices[chunk[2] as usize][1],
                    self.vertices[chunk[2] as usize][2],
                );

                // Möller–Trumbore intersection algorithm
                // https://zhuanlan.zhihu.com/p/451582864
                let edge1 = v1 - v0;
                let edge2 = v2 - v0;
                let s = origin - v0;
                let s1 = direction.cross(edge2);
                let s2 = s.cross(edge1);
                let s1e1 = s1.dot(edge1);

                let t = s2.dot(edge2) / s1e1;
                if t >= min_t {
                    continue;
                }
                let b1 = s1.dot(s) / s1e1;
                let b2 = s2.dot(direction) / s1e1;
                if s1e1.abs() > 1e-6 && t >= 0.0 && b1 >= 0.0 && b2 >= 0.0 && (b1 + b2) <= 1.0 {
                    intersected_face = Some((chunk.try_into().unwrap(), (t * direction + origin).into()));
                    min_t = t;
                }
            }
        }

        intersected_face
    }
}

#[test]
fn test_compute_intersecting_face() {
    let mesh = Mesh {
        vertices: vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ],
        normals: vec![],
        tex_coords: vec![],
        indices: vec![0, 1, 2, 0, 1, 3],
        aabb: AABB{ min: glam::f32::Vec3::from_array([0.0,0.0,0.0]),
             max: glam::f32::Vec3::from_array([1.0,1.0,1.0])}
    };
    let origin = [0.1, 0.1, -1.0];
    let direction = [0.0, 0.0, 1.0];
    let intersected = mesh.compute_intersecting_face(origin, direction);
    assert!(intersected.is_some());
    println!("Intersected face: {:?}", intersected);
}