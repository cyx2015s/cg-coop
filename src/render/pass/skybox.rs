use glium::{Surface, texture::{RawImage2d, Texture2dArray}};
use gltf::camera::Projection;
use glutin::surface::WindowSurface;

use crate::{render::shader::create_program, scene::World};

#[derive(Copy, Clone)]
struct SkyboxVertex {
    position: [f32; 3],
}

glium::implement_vertex!(SkyboxVertex, position);

fn skybox_vertices() -> Vec<SkyboxVertex> {
    let v = [
        // 后
        [-1.0,  1.0, -1.0], [-1.0, -1.0, -1.0], [ 1.0, -1.0, -1.0],
        [ 1.0, -1.0, -1.0], [ 1.0,  1.0, -1.0], [-1.0,  1.0, -1.0],

        // 前
        [-1.0, -1.0,  1.0], [-1.0,  1.0,  1.0], [ 1.0,  1.0,  1.0],
        [ 1.0,  1.0,  1.0], [ 1.0, -1.0,  1.0], [-1.0, -1.0,  1.0],

        // 左
        [-1.0,  1.0,  1.0], [-1.0, -1.0,  1.0], [-1.0, -1.0, -1.0],
        [-1.0, -1.0, -1.0], [-1.0,  1.0, -1.0], [-1.0,  1.0,  1.0],

        // 右
        [ 1.0, -1.0, -1.0], [ 1.0, -1.0,  1.0], [ 1.0,  1.0,  1.0],
        [ 1.0,  1.0,  1.0], [ 1.0,  1.0, -1.0], [ 1.0, -1.0, -1.0],

        // 上
        [-1.0,  1.0, -1.0], [ 1.0,  1.0, -1.0], [ 1.0,  1.0,  1.0],
        [ 1.0,  1.0,  1.0], [-1.0,  1.0,  1.0], [-1.0,  1.0, -1.0],

        // 下
        [-1.0, -1.0, -1.0], [-1.0, -1.0,  1.0], [ 1.0, -1.0, -1.0],
        [ 1.0, -1.0, -1.0], [-1.0, -1.0,  1.0], [ 1.0, -1.0,  1.0],
    ];

    v.iter().map(|&p| SkyboxVertex { position: p }).collect()
}

fn load_skybox_array(
    display: &glium::Display<WindowSurface>
) -> Texture2dArray {
    let faces = [
        "assets/texture/skyboxrt/right.jpg",   // +X
        "assets/texture/skyboxrt/left.jpg",    // -X
        "assets/texture/skyboxrt/top.jpg",     // +Y
        "assets/texture/skyboxrt/bottom.jpg",  // -Y
        "assets/texture/skyboxrt/front.jpg",   // +Z
        "assets/texture/skyboxrt/back.jpg",    // -Z
    ];

    let images: Vec<RawImage2d<u8>> = faces.iter().map(|path| {
        let img = image::open(path).unwrap().to_rgba8();
        let dims = img.dimensions();

        RawImage2d::from_raw_rgba_reversed(
            &img.into_raw(),
            dims,
        )
    }).collect();

    Texture2dArray::new(display, images).unwrap()
}

pub struct SkyboxPass {
    pub program: glium::Program,
    skybox: Texture2dArray,
}

impl SkyboxPass {
    pub fn new(display: &glium::Display<WindowSurface>) -> Self {
        let program = create_program(
            display,
            "assets/shaders/skybox.vert",
            "assets/shaders/skybox.frag");
        let skybox = load_skybox_array(display);
        Self { 
            program,
            skybox,
        }
    }

    pub fn render(&mut self, target: &mut glium::Frame, display: &glium::Display<WindowSurface>, world: &mut World){
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLessOrEqual,
                write: false,
                ..Default::default()
            },
            backface_culling: glium::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let skybox_sampler = glium::uniforms::Sampler::new(&self.skybox)
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
            .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
            .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);
        if let Some(idx) = world.get_selected_camera() {
            let camera = &mut world.cameras[idx].camera;
            let projection = camera.get_projection_matrix();
            let view = camera.get_view_no_translation();
            let uniforms = glium::uniform! {
                projection: projection,
                view: view,
                skybox: skybox_sampler,
            };
            let vertex_data = glium::VertexBuffer::new(display,&skybox_vertices()).unwrap();
            target.draw(
                &vertex_data,
                glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.program,
                &uniforms,
                &params ).unwrap();
        }
        

    }
}