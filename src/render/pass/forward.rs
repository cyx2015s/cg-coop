use crate::render::shader::{ create_program, paths };
use crate::core::vertex::{ Vertex, Normal };
use crate::scene::World;
use crate::scene::light::LightBlock;
use crate::core::material;
use crate::render::scene_renderer::LightSpaceMatrixBlock;
use crate::implement_uniform_block_new;

use glium::Surface;
use glium::Program;
use glium::uniforms::UniformBuffer;
use glium::uniform;
use std::path::Path;
use glium::glutin::surface::WindowSurface;


#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct CascadeZfarsUbo {
    pub cascade_zfars: [f32; 4],
}
implement_uniform_block_new!(CascadeZfarsUbo, cascade_zfars);

pub struct ForwardPass {
    program: Program,
    light_space_matrix_ubo: UniformBuffer<LightSpaceMatrixBlock>,
    light_block_ubo: UniformBuffer<LightBlock>,
    material_ubo: UniformBuffer<material::MaterialBlock>,
    cascade_zfars_ubo: UniformBuffer<CascadeZfarsUbo>,
    default_texture: glium::texture::SrgbTexture2d,
    loaded_texture: Option<glium::texture::SrgbTexture2d>,
}

impl ForwardPass{
    pub fn new(
        display: &glium::Display<WindowSurface>,
        light_space_matrix_ubo: UniformBuffer<LightSpaceMatrixBlock>,
        light_block_ubo: UniformBuffer<LightBlock>,
     ) -> Self {
        let program = create_program(
            display,
            paths::PHONG_VERT,
            paths::PHONG_FRAG,
        );
        let cascade_zfars_block = CascadeZfarsUbo { cascade_zfars: [0.0; 4] };
        let cascade_zfars_ubo = UniformBuffer::new(display, cascade_zfars_block).unwrap();
        let material_block = material::MaterialBlock { material: material::Material::default() };
        let material_ubo = UniformBuffer::new(display, material_block).unwrap();
                let default_texture = {
            let size = 64; // 纹理总大小
            let check_size = 8; // 每个格子的大小 (8x8像素)
            let mut data = Vec::with_capacity(size * size * 4);

            for y in 0..size {
                for x in 0..size {
                    // 根据坐标计算当前像素应该是黑还是白
                    let is_white = ((x / check_size) + (y / check_size)) % 2 == 0;
                    let color = if is_white { 255u8 } else { 0u8 };

                    data.push(color);
                    data.push(color);
                    data.push(color);
                    data.push(255);
                }
            }
            let image =
                glium::texture::RawImage2d::from_raw_rgba_reversed(&data, (size as u32, size as u32));
            glium::texture::SrgbTexture2d::new(display, image).unwrap()
        };

        let loaded_texture: Option<glium::texture::SrgbTexture2d> = {
            let path_jpg = "assets/texture.jpg";
            let path_png = "assets/texture.png";

            let load_path = if Path::new(path_jpg).exists() {
                Some(path_jpg)
            } else if Path::new(path_png).exists() {
                Some(path_png)
            } else {
                None
            };

            if let Some(p) = load_path {
                println!("正在加载纹理: {}", p);
                match image::open(p) {
                    Ok(img) => {
                        let img = img.flipv();
                        let img = img.to_rgba8();
                        let dims = img.dimensions();
                        let raw =
                            glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dims);
                        Some(glium::texture::SrgbTexture2d::new(display, raw).unwrap())
                    }
                    Err(e) => {
                        println!("纹理加载失败: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        };
        Self {
            program,
            light_space_matrix_ubo,
            light_block_ubo,
            material_ubo,
            cascade_zfars_ubo,
            default_texture,
            loaded_texture,
        }
    }

    pub fn render(
        &mut self, 
        display: &glium::Display<WindowSurface>, 
        world: &mut World, 
        shadow_atlas: &glium::texture::DepthTexture2dArray,
        light_block: &LightBlock,
        light_space_matrix: &LightSpaceMatrixBlock,
        target: &mut glium::Frame,
    ) {

        self.light_space_matrix_ubo.write(light_space_matrix);
        self.light_block_ubo.write(light_block);
        if let Some(idx) = world.get_selected_camera() {
            let camera_obj = &mut world.cameras[idx];

            let view = camera_obj.camera.get_view_matrix();
            let perspective = camera_obj.camera.get_projection_matrix();
            let view_pos = camera_obj.camera.get_position();

            let params = glium::draw_parameters::DrawParameters {
                depth: glium::draw_parameters::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    .. Default::default()
                },
                .. Default::default()
            };
            let _point_params = glium::draw_parameters::DrawParameters {
                point_size: Some(5.0),
                ..Default::default()
            };

            // 设置阴影贴图采样器
            let shadow_sampler = glium::uniforms::Sampler::new(shadow_atlas)
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);

            for obj in &world.objects {
                if !obj.visible { continue; }

                let model = obj.transform.get_matrix().to_cols_array_2d();
                let m_block = material::MaterialBlock { material: obj.material };
                self.material_ubo.write(&m_block);

                let vertex_data: Vec<Vertex> = obj.mesh.vertices.iter()
                    .zip(obj.mesh.tex_coords.iter())
                    .map(|(v, t)| Vertex { position: *v, tex_coord: *t })
                    .collect();
                let normal_data: Vec<Normal> = obj.mesh.normals.iter().map(|n| Normal { normal: *n }).collect();

                if vertex_data.is_empty() { continue; }

                let positions = glium::VertexBuffer::new(display, &vertex_data).unwrap();
                let normals = glium::VertexBuffer::new(display, &normal_data).unwrap();

                let indices = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &obj.mesh.indices).unwrap();

                let use_tex = if let Some(tex) = &self.loaded_texture {
                    tex
                } else {
                    &self.default_texture
                };
                let cascadeCount:i32 = 3;
                let splits = crate::render::pass::shadow::ShadowPass::get_cascade_distances(camera_obj.camera.znear, camera_obj.camera.zfar);
                let mut cascade_zfar_block = CascadeZfarsUbo{ cascade_zfars: [0.0; 4]};

                for i in 0..(cascadeCount+1) {
                    cascade_zfar_block.cascade_zfars[i as usize] = splits[i as usize];
                }
                
                self.cascade_zfars_ubo.write(&cascade_zfar_block);
                target.draw(
                    (&positions, &normals),
                    &indices,
                    &self.program,
                    &uniform! { 
                        model: model, 
                        view: view, 
                        perspective: perspective,
                        viewPos: view_pos,
                        cascadeCount: cascadeCount,
                        CascadeZfarsUbo : &self.cascade_zfars_ubo,
                        Material_Block: &self.material_ubo,
                        Light_Block: &self.light_block_ubo,
                        diffuse_tex: use_tex, 
                        has_texture: obj.use_texture,
                        // 传入阴影参数
                        LightSpaceMatrix_Block: &self.light_space_matrix_ubo,
                        shadow_map: shadow_sampler,
                    },
                    &params).unwrap();                
            }

        }
    }
}

