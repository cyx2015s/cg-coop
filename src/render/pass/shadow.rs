use crate::core::math::aabb::AABB;
use crate::core::vertex::Vertex;
use crate::render::scene_renderer::LightSpaceMatrixBlock;
use crate::render::shader::{create_program, paths};
use crate::scene::world::World;

use glium::Program;
use glium::Surface;
use glium::texture::DepthTexture2dArray;
use glium::uniform;
use glutin::surface::WindowSurface;
const CASCADE_COUNT: usize = 3;

#[derive(Copy, Clone)]
pub struct DebugVertex {
    pub position: [f32; 3],
}
glium::implement_vertex!(DebugVertex, position);

#[derive(Clone, Copy)]
pub struct DebugLightBox {
    pub corners: [[f32; 3]; 8],
}

pub struct ShadowPass {
    pub shadow_pass_program: Program,
    pub debug_program: Program,
    pub debug_light_boxes: Vec<DebugLightBox>,
    pub freeze_debug_boxes: bool,
}

impl ShadowPass {
    pub fn new(display: &glium::Display<WindowSurface>) -> Self {
        let shadow_pass_program = create_program(display, paths::SHADOW_VERT, paths::SHADOW_FRAG);
        let debug_program = create_program(
            display,
            "assets/shaders/debug.vert",
            "assets/shaders/debug.frag",
        );
        Self {
            shadow_pass_program,
            debug_program,
            debug_light_boxes: Vec::new(),
            freeze_debug_boxes: false,
        }
    }

    pub fn get_cascade_distances(near: f32, far: f32) -> [f32; CASCADE_COUNT + 1] {
        let lambda = 0.6;
        let mut splits = [0.0; CASCADE_COUNT + 1];
        splits[0] = near;

        for i in 1..CASCADE_COUNT {
            let t = i as f32 / CASCADE_COUNT as f32;
            let uni = near + (far - near) * t;
            let log = near * (far / near).powf(t);
            splits[i] = uni * (1.0 - lambda) + log * lambda;
        }
        splits[CASCADE_COUNT] = far;
        splits
    }

    // =========================
    // Main update
    // =========================
    pub fn update_directional_light_space_matrix(
        &mut self,
        light_space_ubo: &mut LightSpaceMatrixBlock,
        scene: &World,
    ) {
        if self.freeze_debug_boxes {
            return;
        }

        self.debug_light_boxes.clear();
        let scene_box = scene.get_scene_bounding_box();
        let Some(idx) = scene.get_selected_camera() else {
            return;
        };
        let cam_obj = &scene.cameras[idx];
        let cam = &cam_obj.camera;

        let splits = Self::get_cascade_distances(cam.znear, cam.zfar);
        let camera_view = cam.get_view_matrix();

        for (light_index, light_object) in scene.lights.iter().enumerate() {
            if !light_object.light.is_directional() {
                continue;
            }

            let light_dir = light_object.light.direction;

            for cascade in 0..CASCADE_COUNT {
                let (matrix, debug_box) = Self::compute_cascade_light_matrix(
                    camera_view,
                    cam.fovy,
                    cam.aspect,
                    splits[cascade],
                    splits[cascade + 1],
                    light_dir,
                    scene_box,
                );

                let index = light_index * CASCADE_COUNT + cascade;
                light_space_ubo.light_space_matrix[index].matrix = matrix.to_cols_array_2d();

                self.debug_light_boxes.push(debug_box);
            }
        }
    }

    pub fn draw_debug_light_boxes_solid(
        &self,
        target: &mut glium::Frame,
        display: &glium::Display<WindowSurface>,
        world: &World,
    ) {
        let Some(idx) = world.get_selected_camera() else {
            return;
        };
        let camera = &world.cameras[idx].camera;
        for (i, b) in self.debug_light_boxes.iter().enumerate() {
            let (vertices, indices) = cube_solid_from_corners(&b.corners);

            let vbo = glium::VertexBuffer::new(display, &vertices).unwrap();
            let ibo = glium::IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &indices,
            )
            .unwrap();

            let color: [f32; 4] = match i % 3 {
                0 => [1.0, 0.0, 0.0, 0.25],
                1 => [0.0, 1.0, 0.0, 0.25],
                _ => [0.0, 0.0, 1.0, 0.25],
            };

            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: false, // ⭐ 关键
                    ..Default::default()
                },
                backface_culling: glium::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            };

            target
                .draw(
                    &vbo,
                    &ibo,
                    &self.debug_program,
                    &uniform! {
                        view: camera.get_view_matrix(),
                        projection: camera.get_projection_matrix(),
                        u_color: color,
                    },
                    &params,
                )
                .unwrap();
        }
    }

    pub fn get_frustum_corners_world_space(
        camera_view: [[f32; 4]; 4],
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> [[f32; 3]; 8] {
        let projection: glam::f32::Mat4 =
            glam::f32::Mat4::perspective_rh_gl(fov, aspect, near, far);
        let inv: glam::f32::Mat4 =
            (projection * glam::f32::Mat4::from_cols_array_2d(&camera_view)).inverse();
        let mut corners = [[0.0; 3]; 8];
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let pt = inv
                        * glam::f32::Vec4::new(
                            (x as f32) * 2.0 - 1.0,
                            (y as f32) * 2.0 - 1.0,
                            (z as f32) * 2.0 - 1.0,
                            1.0,
                        );
                    corners[x * 4 + y * 2 + z] = (pt.truncate() / pt.w).to_array();
                }
            }
        }
        corners
    }

    fn compute_cascade_light_matrix(
        camera_view: [[f32; 4]; 4],
        fov: f32,
        aspect: f32,
        near: f32,
        far: f32,
        light_dir: [f32; 3],
        scene_box: AABB,
    ) -> (glam::Mat4, DebugLightBox) {
        let frustum_corners =
            Self::get_frustum_corners_world_space(camera_view, fov, aspect, near, far);

        let mut center = glam::Vec3::ZERO;
        for c in frustum_corners.iter() {
            center += glam::Vec3::from(*c);
        }
        center /= 8.0;

        let light_dir = glam::Vec3::from(light_dir).normalize();
        let light_pos = center - light_dir * 100.0;
        let mut up = glam::Vec3::Y;
        if light_dir.x.abs() < 0.001 && light_dir.z.abs() < 0.001 {
            up = glam::Vec3::Z;
        }
        let light_view = glam::Mat4::look_at_rh(light_pos, center, up);

        let mut min = glam::Vec3::splat(f32::INFINITY);
        let mut max = glam::Vec3::splat(f32::NEG_INFINITY);

        for c in frustum_corners.iter() {
            let v = light_view * glam::Vec4::new(c[0], c[1], c[2], 1.0);
            min = min.min(v.truncate());
            max = max.max(v.truncate());
        }

        max = max.max(scene_box.max);
        min = min.min(scene_box.min);

        let z_near_raw = -max.z;
        let z_far_raw = -min.z;

        let padding = (z_far_raw - z_near_raw) * 0.05;

        let z_near = (z_near_raw - padding).max(0.0);
        let z_far = z_far_raw + padding;

        let shadow_map_size = 2048.0;

        let extent_x = max.x - min.x;
        let extent_y = max.y - min.y;

        let texel_size_x = extent_x / shadow_map_size;
        let texel_size_y = extent_y / shadow_map_size;

        min.x = (min.x / texel_size_x).floor() * texel_size_x;
        min.y = (min.y / texel_size_y).floor() * texel_size_y;

        max.x = min.x + shadow_map_size * texel_size_x;
        max.y = min.y + shadow_map_size * texel_size_y;

        let light_proj = glam::Mat4::orthographic_rh_gl(min.x, max.x, min.y, max.y, z_near, z_far);

        let light_space = light_proj * light_view;

        let light_space_corners = [
            [min.x, min.y, -z_near],
            [max.x, min.y, -z_near],
            [max.x, max.y, -z_near],
            [min.x, max.y, -z_near],
            [min.x, min.y, -z_far],
            [max.x, min.y, -z_far],
            [max.x, max.y, -z_far],
            [min.x, max.y, -z_far],
        ];

        let inv_light_view = light_view.inverse();
        let mut world_corners = [[0.0; 3]; 8];

        for (i, c) in light_space_corners.iter().enumerate() {
            let p = inv_light_view * glam::Vec4::new(c[0], c[1], c[2], 1.0);
            world_corners[i] = p.truncate().to_array();
        }

        (
            light_space,
            DebugLightBox {
                corners: world_corners,
            },
        )
    }

    pub fn render(
        &mut self,
        shadow_atlas: &mut DepthTexture2dArray,
        light_space_matrix: &mut LightSpaceMatrixBlock,
        display: &glium::Display<WindowSurface>,
        scene: &World,
    ) {
        self.update_directional_light_space_matrix(light_space_matrix, scene);
        for (light_index, light_object) in scene.lights.iter().enumerate() {
            if !light_object.light.is_directional() {
                for cascade in 0..CASCADE_COUNT {
                    let layer = (light_index * CASCADE_COUNT + cascade) as u32;
                    let depth_layer = shadow_atlas.main_level().layer(layer).unwrap();

                    let mut target =
                        glium::framebuffer::SimpleFrameBuffer::depth_only(display, depth_layer)
                            .unwrap();
                    target.clear_depth(1.0);
                }
            }
            for cascade in 0..CASCADE_COUNT {
                let layer = (light_index * CASCADE_COUNT + cascade) as u32;
                self.render_layer(
                    shadow_atlas,
                    light_space_matrix,
                    display,
                    layer,
                    scene,
                );
            }
        }
    }
    fn render_layer(
        &self,
        shadow_atlas: &mut DepthTexture2dArray,
        light_matrix_ubo: &mut LightSpaceMatrixBlock,
        display: &glium::Display<WindowSurface>,
        layer: u32,
        scene: &World,
    ) {
        // 获取纹理数组的特定层
        let depth_layer = shadow_atlas.main_level().layer(layer).unwrap();

        let mut target =
            glium::framebuffer::SimpleFrameBuffer::depth_only(display, depth_layer).unwrap();
        target.clear_depth(1.0);

        let params = glium::draw_parameters::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            color_mask: (false, false, false, false),
            ..Default::default()
        };

        // 渲染所有网格
        for obj in &scene.objects {
            if !obj.rendering.visible {
                continue;
            }

            let vertices: Vec<Vertex> = obj
                .mesh
                .vertices
                .iter()
                .zip(obj.mesh.tex_coords.iter())
                .map(|(v, t)| Vertex {
                    position: *v,
                    tex_coord: *t,
                    normal: [0.0, 0.0, 0.0],
                })
                .collect();

            if vertices.is_empty() {
                continue;
            }

            let vbo = glium::VertexBuffer::new(display, &vertices).unwrap();
            let ibo = glium::IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &obj.mesh.indices,
            )
            .unwrap();

            let model = obj.transform.get_matrix().to_cols_array_2d();

            let uniforms = uniform! {
                model: model,
                light_space_matrix: light_matrix_ubo.light_space_matrix[layer as usize].matrix,
            };

            target
                .draw(&vbo, &ibo, &self.shadow_pass_program, &uniforms, &params)
                .unwrap();
        }
    }
}

// fn cube_lines_from_corners(c: &[[f32; 3]; 8]) -> Vec<Vertex> {
//     let edges = [
//         (0,1),(1,2),(2,3),(3,0),
//         (4,5),(5,6),(6,7),(7,4),
//         (0,4),(1,5),(2,6),(3,7),
//     ];

//     let mut v = Vec::new();
//     for (a, b) in edges {
//         v.push(Vertex { position: c[a], tex_coord: [0.0, 0.0], normal: [0.0, 0.0, 0.0] });
//         v.push(Vertex { position: c[b], tex_coord: [0.0, 0.0], normal: [0.0, 0.0, 0.0] });
//     }
//     v
// }

fn cube_solid_from_corners(c: &[[f32; 3]; 8]) -> (Vec<DebugVertex>, Vec<u16>) {
    let verts: Vec<DebugVertex> = c.iter().map(|p| DebugVertex { position: *p }).collect();

    let indices: Vec<u16> = vec![
        // front
        0, 1, 2, 2, 3, 0, // back
        5, 4, 7, 7, 6, 5, // left
        4, 0, 3, 3, 7, 4, // right
        1, 5, 6, 6, 2, 1, // top
        3, 2, 6, 6, 7, 3, // bottom
        4, 5, 1, 1, 0, 4,
    ];

    (verts, indices)
}
