use crate::geometry::shape::mesh::{AsMesh, Mesh};
use crate::core::math::aabb::AABB;
// 简单的 B-Spline 基函数计算
fn b_spline_basis(i: usize, k: usize, t: f32, knots: &[f32]) -> f32 {
    if k == 0 {
        if t >= knots[i] && t < knots[i + 1] {
            1.0
        } else {
            0.0
        }
    } else {
        let denom1 = knots[i + k] - knots[i];
        let term1 = if denom1 > 1e-6 {
            ((t - knots[i]) / denom1) * b_spline_basis(i, k - 1, t, knots)
        } else {
            0.0
        };

        let denom2 = knots[i + k + 1] - knots[i + 1];
        let term2 = if denom2 > 1e-6 {
            ((knots[i + k + 1] - t) / denom2) * b_spline_basis(i + 1, k - 1, t, knots)
        } else {
            0.0
        };

        term1 + term2
    }
}

pub struct NurbsSurface {
    pub control_points: Vec<[f32; 3]>,
    pub weights: Vec<f32>,
    pub u_count: usize,
    pub v_count: usize,
    pub degree: usize,
    pub splits: usize,
}

impl AsMesh for NurbsSurface {
    fn as_mesh(&self) -> Mesh {
        let mut aabb = AABB::default();
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();

        let k = self.degree;
        let n_u = self.u_count;
        let n_v = self.v_count;

        // 生成节点向量
        let gen_knots = |n: usize, k: usize| -> Vec<f32> {
            let m = n + k + 1;
            let mut knots = vec![0.0; m];
            for i in 0..m {
                if i < k + 1 {
                    knots[i] = 0.0;
                } else if i >= m - (k + 1) {
                    knots[i] = 1.0;
                } else {
                    knots[i] = (i - k) as f32 / (n - k) as f32;
                }
            }
            knots
        };

        let u_knots = gen_knots(n_u, k);
        let v_knots = gen_knots(n_v, k);
        let step = 1.0 / self.splits as f32;

        // 1. 生成顶点和纹理坐标
        for i in 0..=self.splits {
            let u = (i as f32 * step).clamp(0.0, 0.9999);
            for j in 0..=self.splits {
                let v = (j as f32 * step).clamp(0.0, 0.9999);

                let mut point = [0.0, 0.0, 0.0];
                let mut rational_weight = 0.0;

                for row in 0..n_v {
                    for col in 0..n_u {
                        let idx = row * n_u + col;
                        let nip = b_spline_basis(col, k, u, &u_knots)
                            * b_spline_basis(row, k, v, &v_knots);
                        let w = self.weights[idx];

                        point[0] += self.control_points[idx][0] * nip * w;
                        point[1] += self.control_points[idx][1] * nip * w;
                        point[2] += self.control_points[idx][2] * nip * w;
                        rational_weight += nip * w;
                    }
                }
                point[0] /= rational_weight;
                point[1] /= rational_weight;
                point[2] /= rational_weight;

                vertices.push(point);
                aabb.union_point_array(point);
                tex_coords.push([u, 1.0 - v]);
            }
        }

        // 2. 索引生成与法线计算
        normals.resize(vertices.len(), [0.0, 0.0, 0.0]); // 初始化为0
        let width = self.splits + 1;

        for i in 0..self.splits {
            for j in 0..self.splits {
                let p0_idx = i * width + j; // 当前点
                let p1_idx = i * width + (j + 1); // 下一点 (V方向)
                let p2_idx = (i + 1) * width + j; // 右一点 (U方向)
                let p3_idx = (i + 1) * width + (j + 1); // 右下点

                let p0 = vertices[p0_idx];
                let p_right = vertices[p2_idx]; // U方向
                let p_down = vertices[p1_idx]; // V方向

                let edge_u = [p_right[0] - p0[0], p_right[1] - p0[1], p_right[2] - p0[2]];
                let edge_v = [p_down[0] - p0[0], p_down[1] - p0[1], p_down[2] - p0[2]];

                let normal = [
                    edge_u[1] * edge_v[2] - edge_u[2] * edge_v[1],
                    edge_u[2] * edge_v[0] - edge_u[0] * edge_v[2],
                    edge_u[0] * edge_v[1] - edge_u[1] * edge_v[0],
                ];

                indices.push(p0_idx as u16);
                indices.push(p2_idx as u16);
                indices.push(p1_idx as u16);

                indices.push(p1_idx as u16);
                indices.push(p2_idx as u16);
                indices.push(p3_idx as u16);
                for idx in [p0_idx, p1_idx, p2_idx, p3_idx] {
                    normals[idx][0] += normal[0];
                    normals[idx][1] += normal[1];
                    normals[idx][2] += normal[2];
                }
            }
        }

        // 3. 归一化法线
        for n in &mut normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-6 {
                n[0] /= len;
                n[1] /= len;
                n[2] /= len;
            } else {
                // 防止零向量，默认朝上
                *n = [0.0, 1.0, 0.0];
            }
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
