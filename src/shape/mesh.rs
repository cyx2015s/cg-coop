use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Mesh {
    // 通用的 Mesh 数据字段：位置、法线、索引
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// 每个三角形 3 个索引
    pub indices: Vec<u16>,
}

pub trait AsMesh {
    fn as_mesh(&self) -> Mesh;
}

impl Mesh {
    /// 导出为 OBJ 文件
    pub fn save_obj<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;

        // 1. 写入版本/注释头
        writeln!(file, "# Exported by CG-Coop")?;

        // 2. 写入顶点 (v x y z)
        for v in &self.vertices {
            writeln!(file, "v {} {} {}", v[0], v[1], v[2])?;
        }

        // 3. 写入法线 (vn x y z)
        for n in &self.normals {
            writeln!(file, "vn {} {} {}", n[0], n[1], n[2])?;
        }

        // 4. 写入面 (f v1//vn1 v2//vn2 v3//vn3)
        // OBJ 索引是从 1 开始的
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
        
        // 配置加载选项：
        // triangulate: 自动把四边形面切成三角形
        // single_index: 自动处理顶点去重，保证位置和法线共用同一个索引
        let load_options = tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        };

        let (models, _materials) = tobj::load_obj(path, &load_options)
            .map_err(|e| format!("Failed to load OBJ: {}", e))?;

        // 只取第一个模型 (假设 OBJ 里只有一个物体)
        if let Some(model) = models.first() {
            let mesh = &model.mesh;

            // 1. 转换顶点格式 (Vec<f32> -> Vec<[f32; 3]>)
            let vertices: Vec<[f32; 3]> = mesh.positions.chunks(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();

            // 2. 转换法线格式
            let normals: Vec<[f32; 3]> = if mesh.normals.is_empty() {
                // 如果 OBJ 没法线，填一个默认的向上法线
                vec![[0.0, 1.0, 0.0]; vertices.len()]
            } else {
                mesh.normals.chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect()
            };

            // 3. 转换索引 (u32 -> u16)
            // 注意：如果模型顶点数超过 65535，这里会溢出
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