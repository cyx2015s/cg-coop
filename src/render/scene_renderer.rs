use super::pass::{DebugPass, ForwardPass, QuadPass, ShadowPass};

use crate::implement_uniform_block_new;
use crate::render::pass::SkyboxPass;
use crate::scene::World;
use crate::scene::light::{Light, LightBlock};

use glium::texture::DepthTexture2dArray;
use glutin::surface::WindowSurface;
use glium::implement_vertex;

const SHADOW_SIZE: u32 = 2048;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct LightSpaceMatrix {
    pub matrix: [[f32; 4]; 4],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct LightSpaceMatrixBlock {
    pub light_space_matrix: [LightSpaceMatrix; 128],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct SpotLightSpaceMatrixBlock {
    pub spot_light_space_matrix: [LightSpaceMatrix; 32],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct PointLightSpaceMatrixBlock {
    pub point_light_space_matrix: [LightSpaceMatrix; 256],
}

implement_uniform_block_new!(LightSpaceMatrix, matrix);
implement_uniform_block_new!(LightSpaceMatrixBlock, light_space_matrix);
implement_uniform_block_new!(SpotLightSpaceMatrixBlock, spot_light_space_matrix);
implement_uniform_block_new!(PointLightSpaceMatrixBlock, point_light_space_matrix);

pub struct SceneRenderer {
    pub shadow_atlas: DepthTexture2dArray,
    pub spot_shadow_atlas: DepthTexture2dArray,
    pub point_shadow_atlas: DepthTexture2dArray,
    pub light_matrix_block: LightSpaceMatrixBlock,
    pub spot_light_matrix_block: SpotLightSpaceMatrixBlock,
    pub point_light_matrix_block: PointLightSpaceMatrixBlock,
    pub light_block: LightBlock,
    pub shadow_pass: ShadowPass,
    pub forward_pass: ForwardPass,
    pub quad_pass: QuadPass,
    pub debug_pass: DebugPass,
    pub skybox_pass: SkyboxPass,
}

impl SceneRenderer {
    pub fn new(display: &glium::Display<WindowSurface>) -> Self {
        // 创建阴影贴图
        let shadow_atlas = DepthTexture2dArray::empty_with_format(
            display,
            glium::texture::DepthFormat::F32,
            glium::texture::MipmapsOption::NoMipmap,
            SHADOW_SIZE,
            SHADOW_SIZE,
            96_u32,
        )
        .unwrap();
        let spot_shadow_atlas = DepthTexture2dArray::empty_with_format(
            display,
            glium::texture::DepthFormat::F32,
            glium::texture::MipmapsOption::NoMipmap,
            SHADOW_SIZE,
            SHADOW_SIZE,
            32_u32,
        )
        .unwrap();
        let point_shadow_atlas = DepthTexture2dArray::empty_with_format(
            display,
            glium::texture::DepthFormat::F32,
            glium::texture::MipmapsOption::NoMipmap,
            SHADOW_SIZE,
            SHADOW_SIZE,
            196_u32,
        )
        .unwrap();

        let light_matrix_block = LightSpaceMatrixBlock {
            light_space_matrix: [LightSpaceMatrix {
                matrix: [[0.0; 4]; 4],
            }; 128],
        };
        let spot_light_matrix_block = SpotLightSpaceMatrixBlock {
            spot_light_space_matrix: [LightSpaceMatrix {
                matrix: [[0.0; 4]; 4],
            }; 32],
        };

        let point_light_matrix_block = PointLightSpaceMatrixBlock {
            point_light_space_matrix: [LightSpaceMatrix {
                matrix: [[0.0; 4]; 4],
            }; 256],
        };

        let light_space_matrix_ubo =
            glium::uniforms::UniformBuffer::new(display, light_matrix_block).unwrap();
        let spot_light_space_matrix_ubo =
            glium::uniforms::UniformBuffer::new(display, spot_light_matrix_block).unwrap();
        let point_light_space_matrix_ubo =
            glium::uniforms::UniformBuffer::new(display, point_light_matrix_block).unwrap();
        let light_block = LightBlock {
            lights: [Light::NONE; 32],
            num_lights: 0,
        };

        let light_block_ubo = glium::uniforms::UniformBuffer::new(display, light_block).unwrap();
       
        Self {
            shadow_atlas,
            spot_shadow_atlas,
            point_shadow_atlas,
            light_matrix_block,
            spot_light_matrix_block,
            point_light_matrix_block,
            light_block,
            shadow_pass: ShadowPass::new(display),
            forward_pass: ForwardPass::new(display, light_space_matrix_ubo, spot_light_space_matrix_ubo, point_light_space_matrix_ubo, light_block_ubo),
            quad_pass: QuadPass::new(display),
            debug_pass: DebugPass::new(display),
            skybox_pass: SkyboxPass::new(display),
        }
    }

    pub fn render(
        &mut self,
        display: &glium::Display<WindowSurface>,
        world: &mut World,
        target: &mut glium::Frame,
    ) {
        let (width, height) = display.get_framebuffer_dimensions();
        let aspect = width as f32 / height as f32;
        if let Some(idx) = world.get_selected_camera() {
            let camera = &mut world.cameras[idx].camera;
            camera.aspect = aspect;
        }
        self.skybox_pass.render(target, display, world);

        self.shadow_pass.render(
            &mut self.shadow_atlas,
            &mut self.spot_shadow_atlas,
            &mut self.point_shadow_atlas,
            &mut self.light_matrix_block,
            &mut self.spot_light_matrix_block,
            &mut self.point_light_matrix_block,
            display,
            world,
        );

        self.light_block.num_lights = 0;
        for (idx, light_obj) in world.lights.iter().enumerate() {
            self.light_block.lights[idx] = light_obj.light;
            self.light_block.num_lights += 1;
        }
        self.shadow_pass.freeze_debug_boxes = world.debug_frustum;

        if world.debug {
            self.quad_pass
                .render(target, &self.shadow_atlas, world.layer);
        } else {
            self.forward_pass.render(
                display,
                world,
                &self.shadow_atlas,
                &self.spot_shadow_atlas,
                &self.point_shadow_atlas,
                &self.light_block,
                &self.light_matrix_block,
                &self.spot_light_matrix_block,
                &self.point_light_matrix_block,
                target,
            );
        }
        self.debug_pass.render(target, display, world);

        // self.shadow_pass.draw_debug_light_boxes_solid(target, display, world);
        
        self.render_weapon(display, world, target);
    }
    
    // 渲染第一人称武器
    fn render_weapon(
        &self,
        display: &glium::Display<WindowSurface>,
        world: &World,
        target: &mut glium::Frame,
    ) {
        use glium::Surface;
        
        // 定义Vertex结构体
        #[derive(Copy, Clone)]
        struct WeaponVertex {
            position: [f32; 3],
            normal: [f32; 3],
        }
        
        implement_vertex!(WeaponVertex, position, normal);
        
        if let Some(cam_idx) = world.get_selected_camera() {
            let camera = &world.cameras[cam_idx].camera;
            
            if let Some(weapon_mesh) = &camera.weapon_mesh {
                // 清除深度缓冲，让武器始终显示在最前
                target.clear_depth(1.0);
                
                // 计算武器在世界空间的变换
                let cam_pos = camera.transform.position;
                let cam_rot = camera.transform.rotation;
                
                // 武器位置 = 相机位置 + 旋转后的偏移
                let local_offset = camera.weapon_transform.position;
                let world_offset = cam_rot * local_offset;
                let weapon_pos = cam_pos + world_offset;
                
                // 武器的旋转跟随相机
                let weapon_rot = cam_rot * camera.weapon_transform.rotation;
                
                // 构建模型矩阵
                let model_matrix = glam::Mat4::from_scale_rotation_translation(
                    camera.weapon_transform.scale,
                    weapon_rot,
                    weapon_pos,
                );
                
                // 准备渲染参数
                let view_matrix = camera.get_view_matrix();
                let projection_matrix = camera.get_projection_matrix();
                
                let uniforms = glium::uniform! {
                    model: model_matrix.to_cols_array_2d(),
                    view: view_matrix,
                    projection: projection_matrix,
                    // 简单的固定光照，不受场景光影响
                    lightPos: [0.0f32, 5.0, 0.0],
                    lightColor: [1.0f32, 1.0, 1.0],
                    objectColor: [0.3f32, 0.3, 0.3],
                };
                
                let params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    backface_culling: glium::BackfaceCullingMode::CullClockwise,
                    ..Default::default()
                };
                
                // 简单的顶点着色器
                let vertex_shader_src = r#"
                    #version 330 core
                    in vec3 position;
                    in vec3 normal;
                    
                    uniform mat4 model;
                    uniform mat4 view;
                    uniform mat4 projection;
                    
                    out vec3 FragPos;
                    out vec3 Normal;
                    
                    void main() {
                        FragPos = vec3(model * vec4(position, 1.0));
                        Normal = mat3(transpose(inverse(model))) * normal;
                        gl_Position = projection * view * vec4(FragPos, 1.0);
                    }
                "#;
                
                let fragment_shader_src = r#"
                    #version 330 core
                    in vec3 FragPos;
                    in vec3 Normal;
                    
                    uniform vec3 lightPos;
                    uniform vec3 lightColor;
                    uniform vec3 objectColor;
                    
                    out vec4 FragColor;
                    
                    void main() {
                        // 简单的Phong光照
                        vec3 norm = normalize(Normal);
                        vec3 lightDir = normalize(lightPos - FragPos);
                        float diff = max(dot(norm, lightDir), 0.0);
                        vec3 diffuse = diff * lightColor;
                        
                        vec3 ambient = 0.3 * lightColor;
                        vec3 result = (ambient + diffuse) * objectColor;
                        
                        FragColor = vec4(result, 1.0);
                    }
                "#;
                
                let program = glium::Program::from_source(
                    display,
                    vertex_shader_src,
                    fragment_shader_src,
                    None,
                ).unwrap();
                
                // 转换Mesh数据为glium::Vertex格式
                let vertices: Vec<WeaponVertex> = weapon_mesh.vertices.iter()
                    .zip(weapon_mesh.normals.iter())
                    .map(|(pos, normal)| WeaponVertex {
                        position: *pos,
                        normal: *normal,
                    })
                    .collect();
                
                // 创建顶点缓冲
                let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
                let index_buffer = glium::IndexBuffer::new(
                    display,
                    glium::index::PrimitiveType::TrianglesList,
                    &weapon_mesh.indices,
                ).unwrap();
                
                target.draw(
                    &vertex_buffer,
                    &index_buffer,
                    &program,
                    &uniforms,
                    &params,
                ).unwrap();
            }
        }
    }
}
