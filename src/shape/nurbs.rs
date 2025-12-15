use crate::shape::mesh::{AsMesh, Mesh};

/// NURBS 曲面定义
pub struct NurbsSurface {
    /// 控制点网格 (扁平化存储: index = u * v_count + v)
    pub control_points: Vec<[f32; 3]>,
    /// 每个控制点的权重
    pub weights: Vec<f32>,
    
    /// U 方向控制点数量
    pub u_count: usize,
    /// V 方向控制点数量
    pub v_count: usize,
    
    /// 阶数 (Degree)
    pub degree: usize,
    
    /// 渲染精度
    pub splits: u32,
}

impl NurbsSurface {
    /// 创建一个标准的“开放均匀”节点矢量
    fn generate_knots(n: usize, p: usize) -> Vec<f32> {
        let m = n + p + 1; // 节点总数
        let mut knots = Vec::with_capacity(m);

        // 开头 p+1 个 0.0
        for _ in 0..=p {
            knots.push(0.0);
        }

        // 中间均匀分布
        let count = n - p; // 中间段的数量
        for i in 1..count {
            knots.push(i as f32 / count as f32);
        }

        // 结尾 p+1 个 1.0
        for _ in 0..=p {
            knots.push(1.0);
        }
        
        knots
    }

    /// Cox-de Boor 递归算法核心：计算基函数
    /// i: 节点索引, p: 阶数, u: 参数, knots: 节点向量
    fn basis_func(i: usize, p: usize, u: f32, knots: &[f32]) -> f32 {
        if p == 0 {
            if u >= knots[i] && u < knots[i + 1] {
                return 1.0;
            } else {
                // 特殊处理 u=1.0 的情况，保证闭合
                if (u - 1.0).abs() < 1e-5 && (knots[i+1] - 1.0).abs() < 1e-5 {
                    return 1.0;
                }
                return 0.0;
            }
        }

        let u_i = knots[i];
        let u_ip1 = knots[i + 1];
        let u_ip = knots[i + p];
        let u_ip1p = knots[i + p + 1];

        let term1 = if (u_ip - u_i).abs() < 1e-6 {
            0.0
        } else {
            ((u - u_i) / (u_ip - u_i)) * Self::basis_func(i, p - 1, u, knots)
        };

        let term2 = if (u_ip1p - u_ip1).abs() < 1e-6 {
            0.0
        } else {
            ((u_ip1p - u) / (u_ip1p - u_ip1)) * Self::basis_func(i + 1, p - 1, u, knots)
        };

        term1 + term2
    }

    /// 计算曲面上 (u, v) 处的点坐标
    fn evaluate(&self, u: f32, v: f32, knots_u: &[f32], knots_v: &[f32]) -> [f32; 3] {
        let mut numerator = [0.0, 0.0, 0.0]; // 分子
        let mut denominator = 0.0;           // 分母

        // 遍历所有控制点 
        for i in 0..self.u_count {
            // 计算 U 方向基函数 N_i(u)
            let nip_u = Self::basis_func(i, self.degree, u, knots_u);
            
            // 如果基函数为 0 (意味着这个控制点太远，不影响当前位置)，直接跳过
            if nip_u.abs() < 1e-6 { continue; }

            for j in 0..self.v_count {
                // 计算 V 方向基函数 N_j(v)
                let nip_v = Self::basis_func(j, self.degree, v, knots_v);
                
                if nip_v.abs() < 1e-6 { continue; }

                let idx = i * self.v_count + j;
                let weight = self.weights[idx];
                let pt = self.control_points[idx];

                // 核心公式: P(u,v) = Σ (Ni * Nj * Wij * Pij) / Σ (Ni * Nj * Wij)
                let factor = nip_u * nip_v * weight;

                numerator[0] += pt[0] * factor;
                numerator[1] += pt[1] * factor;
                numerator[2] += pt[2] * factor;
                denominator += factor;
            }
        }

        if denominator.abs() < 1e-6 {
            return [0.0, 0.0, 0.0];
        }

        [
            numerator[0] / denominator,
            numerator[1] / denominator,
            numerator[2] / denominator,
        ]
    }
}

impl AsMesh for NurbsSurface {
    fn as_mesh(&self) -> Mesh {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        // 1. 生成节点向量 
        let knots_u = Self::generate_knots(self.u_count, self.degree);
        let knots_v = Self::generate_knots(self.v_count, self.degree);

        let step_u = 1.0 / self.splits as f32;
        let step_v = 1.0 / self.splits as f32;

        // 2. 采样点生成
        for i in 0..=self.splits {
            let u = (i as f32 * step_u).min(1.0); // 确保不超过 1.0
            for j in 0..=self.splits {
                let v = (j as f32 * step_v).min(1.0);

                let p = self.evaluate(u, v, &knots_u, &knots_v);
                vertices.push(p);

                // 计算法线 (中心差分法)
                let delta = 0.005;
                let u_next = (u + delta).min(1.0);
                let v_next = (v + delta).min(1.0);
                
                let p_du = self.evaluate(u_next, v, &knots_u, &knots_v);
                let p_dv = self.evaluate(u, v_next, &knots_u, &knots_v);

                let tangent_u = [p_du[0] - p[0], p_du[1] - p[1], p_du[2] - p[2]];
                let tangent_v = [p_dv[0] - p[0], p_dv[1] - p[1], p_dv[2] - p[2]];

                // Cross Product
                let nx = tangent_u[1] * tangent_v[2] - tangent_u[2] * tangent_v[1];
                let ny = tangent_u[2] * tangent_v[0] - tangent_u[0] * tangent_v[2];
                let nz = tangent_u[0] * tangent_v[1] - tangent_u[1] * tangent_v[0];
                
                // Normalize
                let len = (nx * nx + ny * ny + nz * nz).sqrt();
                if len > 1e-6 {
                    normals.push([nx / len, ny / len, nz / len]);
                } else {
                    normals.push([0.0, 1.0, 0.0]);
                }
            }
        }

        // 3. 生成索引
        let cols = self.splits + 1;
        for i in 0..self.splits {
            for j in 0..self.splits {
                let current = i * cols + j;
                let next = current + 1;
                let bottom = current + cols;
                let bottom_next = bottom + 1;

                // Triangle 1
                indices.push(current as u16);
                indices.push(bottom as u16); 
                indices.push(next as u16);

                // Triangle 2
                indices.push(next as u16);
                indices.push(bottom as u16);
                indices.push(bottom_next as u16);
            }
        }

        Mesh { vertices, normals, indices }
    }
}