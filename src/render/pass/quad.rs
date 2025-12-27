use glium::implement_vertex;
use glium::index::{NoIndices, PrimitiveType};
use glium::vertex::VertexBuffer;
use glium::{Program, Surface};

#[derive(Copy, Clone)]
pub struct FullscreenVertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
}

implement_vertex!(FullscreenVertex, position, tex_coord);

pub struct QuadPass {
    pub vbo: VertexBuffer<FullscreenVertex>,
    pub indices: NoIndices,
    pub program: Program,
}

impl QuadPass {
    pub fn new(display: &impl glium::backend::Facade) -> Self {
        let vertices = [
            FullscreenVertex {
                position: [-1.0, 1.0],
                tex_coord: [0.0, 1.0],
            },
            FullscreenVertex {
                position: [-1.0, -1.0],
                tex_coord: [0.0, 0.0],
            },
            FullscreenVertex {
                position: [1.0, -1.0],
                tex_coord: [1.0, 0.0],
            },
            FullscreenVertex {
                position: [-1.0, 1.0],
                tex_coord: [0.0, 1.0],
            },
            FullscreenVertex {
                position: [1.0, -1.0],
                tex_coord: [1.0, 0.0],
            },
            FullscreenVertex {
                position: [1.0, 1.0],
                tex_coord: [1.0, 1.0],
            },
        ];
        let program = crate::render::shader::create_program(
            display,
            "assets/shaders/quad.vert",
            "assets/shaders/quad.frag",
        );

        let vbo = VertexBuffer::new(display, &vertices).unwrap();
        let indices = NoIndices(PrimitiveType::TrianglesList);

        Self {
            vbo,
            indices,
            program,
        }
    }

    pub fn render(
        &self,
        target: &mut glium::Frame,
        shadow_atlas: &glium::texture::DepthTexture2dArray,
        layer: usize,
    ) {
        let shadow_sampler = glium::uniforms::Sampler::new(shadow_atlas)
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
            .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
            .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);
        let params = glium::draw_parameters::DrawParameters {
            depth: glium::draw_parameters::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let uniforms = glium::uniform! {
            shadow_map: shadow_sampler,
            layer: layer as i32,
        };
        target
            .draw(&self.vbo, &self.indices, &self.program, &uniforms, &params)
            .unwrap();
    }
}
