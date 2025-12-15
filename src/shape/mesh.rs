use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u16>,
}

pub trait AsMesh {
    fn as_mesh(&self) -> Mesh;
}

impl Mesh {
    /// 导出为 OBJ 文件
    pub fn save_obj<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        writeln!(file, "# Exported by CG-Coop")?;

        for v in &self.vertices {
            writeln!(file, "v {} {} {}", v[0], v[1], v[2])?;
        }

        for n in &self.normals {
            writeln!(file, "vn {} {} {}", n[0], n[1], n[2])?;
        }

        for chunk in self.indices.chunks(3) {
            if chunk.len() == 3 {
                let i0 = chunk[0] + 1;
                let i1 = chunk[1] + 1;
                let i2 = chunk[2] + 1;
                writeln!(file, "f {}//{} {}//{} {}//{}", i0, i0, i1, i1, i2, i2)?;
            }
        }
        
        println!("Mesh saved successfully.");
        Ok(())
    }

    /// 从 OBJ 文件导入
    pub fn load_obj<P: AsRef<Path>>(path: P) -> Result<Mesh, String> {
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

            let vertices: Vec<[f32; 3]> = mesh.positions.chunks(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();

            // 智能法线计算 
            let normals: Vec<[f32; 3]> = if mesh.normals.is_empty() {
                println!("OBJ 模型缺少法线，正在自动计算平滑法线...");
                compute_smooth_normals(&vertices, &mesh.indices)
            } else {
                mesh.normals.chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect()
            };

            let indices: Vec<u16> = mesh.indices.iter()
                .map(|&i| i as u16)
                .collect();

            println!("Loaded OBJ: {} vertices, {} indices", vertices.len(), indices.len());

            Ok(Mesh {
                vertices,
                normals,
                indices,
            })
        } else {
            Err("OBJ file contains no models".to_string())
        }
    }
}

// 辅助函数：计算平滑法线
fn compute_smooth_normals(vertices: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0, 0.0, 0.0]; vertices.len()];

    // 1. 遍历每个三角形，计算面法线并累加到顶点
    for chunk in indices.chunks(3) {
        if chunk.len() == 3 {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            let v0 = vertices[i0];
            let v1 = vertices[i1];
            let v2 = vertices[i2];

            // 向量减法
            let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
            let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

            // 叉乘 (Cross Product) 计算法线
            let normal = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];

            // 累加到三个顶点上
            for idx in [i0, i1, i2] {
                normals[idx][0] += normal[0];
                normals[idx][1] += normal[1];
                normals[idx][2] += normal[2];
            }
        }
    }

    // 2. 归一化每个法线
    for n in &mut normals {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 0.0 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        } else {
            // 防止除以0，给个默认上方向
            *n = [0.0, 1.0, 0.0]; 
        }
    }

    normals
}