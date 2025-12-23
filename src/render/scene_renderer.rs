use super::pass::{ShadowPass, ForwardPass, QuadPass, DebugPass};

use crate::scene::light::{LightBlock, Light};
use crate::scene::World;
use crate::implement_uniform_block_new;

use glium::uniform;
use glutin::surface::WindowSurface;
use glium::texture::DepthTexture2dArray;



const CASCADE_COUNT: usize = 3;
const MAX_LIGHTS: usize = 32;
const MAX_CASCADES: usize = MAX_LIGHTS * CASCADE_COUNT;
const SHADOW_SIZE: u32 = 2048;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct LightSpaceMatrix {
    pub matrix: [[f32; 4]; 4],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct LightSpaceMatrixBlock{
    pub light_space_matrix: [LightSpaceMatrix; 128],
}

implement_uniform_block_new!(LightSpaceMatrix, matrix);
implement_uniform_block_new!(LightSpaceMatrixBlock, light_space_matrix);

pub struct SceneRenderer {
    pub shadow_atlas: DepthTexture2dArray,
    pub light_matrix_block: LightSpaceMatrixBlock,
    pub light_block: LightBlock,
    pub shadow_pass: ShadowPass,
    pub forward_pass: ForwardPass,
    pub quad_pass: QuadPass,
    pub debug_pass: DebugPass,
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
            128 as u32,
        ).unwrap();

        let light_matrix_block = LightSpaceMatrixBlock{
            light_space_matrix: [LightSpaceMatrix{ matrix: [[0.0;4];4] }; 128],
        };
            
        
        let light_space_matrix_ubo = glium::uniforms::UniformBuffer::new(
                display,
                light_matrix_block
        ).unwrap();
        
        let light_block = LightBlock{
            lights: [Light::new(); 32],
            num_lights: 0,
        };

        let light_block_ubo = glium::uniforms::UniformBuffer::new(
                display,
                light_block
        ).unwrap();
        
        Self {
            shadow_atlas,
            light_matrix_block,
            light_block,
            shadow_pass: ShadowPass::new(display),
            forward_pass: ForwardPass::new(display, light_space_matrix_ubo, light_block_ubo),
            quad_pass: QuadPass::new(display),
            debug_pass: DebugPass::new(display),
        }
    }

    pub fn render(&mut self, display: &glium::Display<WindowSurface>, world: &mut World, target: &mut glium::Frame) {

        let (width, height) = display.get_framebuffer_dimensions();
        let aspect = width as f32 / height as f32;
        if let Some(idx) = world.get_selected_camera() {
            let camera = &mut world.cameras[idx].camera;
            camera.aspect = aspect;
        }
        
        self.shadow_pass.render(&mut self.shadow_atlas, &mut self.light_matrix_block, display, world);
        
        self.light_block.num_lights = 0;
        for (idx, light_obj) in world.lights.iter().enumerate() {
            self.light_block.lights[idx] = light_obj.light;
            self.light_block.num_lights += 1;
        }
        if world.debug_frustum {
            self.shadow_pass.freeze_debug_boxes = true;
        } else{
            self.shadow_pass.freeze_debug_boxes = false;
        }

        if world.debug {
            self.quad_pass.render(target, &self.shadow_atlas, world.layer);
        } else {
            self.forward_pass.render(display, world, &self.shadow_atlas, &self.light_block, &self.light_matrix_block, target);
        }
        self.debug_pass.render(target, display, world);

        // self.shadow_pass.draw_debug_light_boxes_solid(target, display, world);
    }

}