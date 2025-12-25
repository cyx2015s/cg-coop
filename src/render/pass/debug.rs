use std::any::{Any, TypeId};

use glium::index::PrimitiveType;
use glium::uniform;
use glium::{Program, Surface};
use glutin::surface::WindowSurface;

use crate::core::vertex::Vertex;
use crate::geometry::shape::cube::Cube;
use crate::render::shader::{create_program, paths};
use crate::scene::world::ShapeKind;
use crate::scene::world::World;

const CASCADE_COUNT: usize = 3;

pub struct DebugPass {
    program: Program,
}

impl DebugPass {
    pub fn new(display: &glium::Display<WindowSurface>) -> Self {
        let program = create_program(
            display,
            "assets/shaders/debug.vert",
            "assets/shaders/debug.frag",
        );

        Self { program }
    }

    pub fn render(
        &self,
        target: &mut glium::Frame,
        display: &glium::Display<WindowSurface>,
        scene: &mut World,
    ) {
        let Some(cam_idx) = scene.get_selected_camera() else {
            return;
        };
        let cam_obj = &scene.cameras[cam_idx];
        let cam = &cam_obj.camera;

        let view = cam.get_view_matrix();
        let perspective = glam::Mat4::perspective_rh_gl(cam.fovy, cam.aspect, cam.znear, cam.zfar);

        if let Some(obj) = scene.get_selected_mut() {
            if let ShapeKind::Nurbs {
                control_points,
                current_nurbs_idx,
                ..
            } = &obj.kind
            {
                let obj_matrix = obj.transform.get_matrix();
                let uniforms = uniform! {
                    model: obj_matrix.to_cols_array_2d(),
                    view: view,
                    projection: perspective.to_cols_array_2d(),
                    selected_idx: *current_nurbs_idx as i32,
                };
                let params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::Overwrite,
                        write: false,
                        ..Default::default()
                    },
                    point_size: Some(10.0),
                    ..Default::default()
                };
                let vertex_data: Vec<Vertex> = control_points
                    .iter()
                    .map(|v| Vertex {
                        position: *v,
                        tex_coord: [0.0; 2],
                        normal: [0.0; 3],
                    })
                    .collect();
                let debug_vbo = glium::vertex::VertexBuffer::new(display, &vertex_data).unwrap();
                target
                    .draw(
                        &debug_vbo,
                        &glium::index::NoIndices(glium::index::PrimitiveType::Points),
                        &self.program,
                        &uniforms,
                        &params,
                    )
                    .unwrap();
            }
            if let ShapeKind::Imported {} = &obj.kind
                && let Some(selected) = obj.selected_vertex_index
            {
                let debug_vertex = obj.mesh.vertices[selected];
                let obj_matrix = obj.transform.get_matrix();

                let uniforms = uniform! {
                    model: obj_matrix.to_cols_array_2d(),
                    view: view,
                    projection: perspective.to_cols_array_2d(),
                    selected_idx: 0,
                };
                let params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::Overwrite,
                        write: false,
                        ..Default::default()
                    },
                    point_size: Some(10.0),
                    ..Default::default()
                };
                let vertex_data: Vec<Vertex> = vec![Vertex {
                    position: debug_vertex,
                    tex_coord: [0.0; 2],
                    normal: [0.0; 3],
                }];
                let debug_vbo = glium::vertex::VertexBuffer::new(display, &vertex_data).unwrap();
                target
                    .draw(
                        &debug_vbo,
                        &glium::index::NoIndices(glium::index::PrimitiveType::Points),
                        &self.program,
                        &uniforms,
                        &params,
                    )
                    .unwrap();
                // println!("Selected vertex position: {:?}", debug_vertex);
            }
        }
    }
}
