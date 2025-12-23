use glium::{Program, Surface};
use glium::index::PrimitiveType;
use glium::uniform;
use glutin::surface::WindowSurface;

use crate::scene::world::ShapeKind;
use crate::core::vertex::Vertex;
use crate::scene::world::World;
use crate::render::shader::{create_program, paths};

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
        let Some(cam_idx) = scene.get_selected_camera() else { return; };
        let cam_obj = &scene.cameras[cam_idx];
        let cam = &cam_obj.camera;

        let view = cam.get_view_matrix();
        let perspective = glam::Mat4::perspective_rh_gl(
            cam.fovy,
            cam.aspect,
            cam.znear,
            cam.zfar,
        );

        if let Some(obj) = scene.get_selected_mut() {
             if let ShapeKind::Nurbs { control_points, current_nurbs_idx, .. } = &obj.kind {
                let obj_matrix = obj.transform.get_matrix(); 
                let uniforms = uniform! {
                    model: obj_matrix.to_cols_array_2d(),
                    view: view,
                    projection: perspective.to_cols_array_2d(),
                    selected_idx: *current_nurbs_idx as i32,
                };
                let params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLess,
                        write: false,
                        ..Default::default()
                    },
                    point_size: Some(10.0),
                    ..Default::default()
                };
                let vertex_data: Vec<Vertex> = control_points.iter()
                    .map(|v| Vertex { position: *v, tex_coord: [0.0; 2] })
                    .collect();
                let debug_vbo = glium::vertex::VertexBuffer::new(display, &vertex_data).unwrap();
                target.draw(
                    &debug_vbo,
                    &glium::index::NoIndices(glium::index::PrimitiveType::Points),
                    &self.program, &uniforms,
                &params).unwrap();
            }
        }

    }
}
