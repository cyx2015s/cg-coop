use std::time::Instant;

use cg_coop::render::SceneRenderer;
use cg_coop::scene::World;
use cg_coop::ui::{ UIBuild, UIHandle};



use imgui::Condition;
use glium::winit::event::{DeviceEvent, Event, WindowEvent};
use glium::*;
fn main() {
    

    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
    .with_title("Project - Mini Blender Mode")
    .with_inner_size(1200, 900)
    .build(&event_loop);

    let mut global_ctx = cg_coop::ui::ctx::GlobalContext::new(&display, &window);
    let mut last_frame = Instant::now();
    let mut scene_renderer = SceneRenderer::new(&display);
    let mut scene = World::new();
    scene.init_scene_1(&display);

    #[allow(deprecated)]
    event_loop.run(move |ev, window_target| {
        global_ctx.handle_event(&ev, &window);
        match ev {
            Event::DeviceEvent { device_id, event } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
                        
                        scene.handle_mouse_move(delta, &window);
                    },
                    _ => {}
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => {
                    
                    global_ctx.update_time();
                    let mut target = display.draw();
                    target.clear_color_and_depth((0.05, 0.05, 0.1, 1.0), 1.0);
                    let ui = global_ctx.ui_ctx.frame();
                    let _cn_font = ui.push_font(global_ctx.cn_font);

                    ui.show_demo_window(&mut true); 

                    ui.window("操作说明")
                        .size([300.0, 100.0], Condition::FirstUseEver)
                        .build(|| {
                            ui.text_wrapped("按V键漫游\n按B键环绕\n滚轮放大缩小视角\n按R恢复视角\n按P键截图\n按WS在摄像头方向前后移动\n按AD左右移动\n按Space/Ctrl上升下降");     
                        });

                    scene.build_ui(ui);

                    target.clear_color_and_depth((0.05, 0.05, 0.1, 1.0), 1.0); 
                    let now = Instant::now();
                    let dt = (now - last_frame).as_secs_f32();
                    last_frame = now;
                    scene.step(dt);
                    scene_renderer.render(&display, &mut scene,&mut target);
                    

                    _cn_font.pop();
                    scene.handle_ui_input(ui, &display);
                    let draw_data = global_ctx.ui_ctx.render();
                    if draw_data.draw_lists_count() > 0 {
                        global_ctx.ui_renderer.render(&mut target, draw_data).unwrap();
                    }
                    target.finish().unwrap();

                }
                winit::event::WindowEvent::CloseRequested => { window_target.exit(); }
                _ => (),
            },
            Event::AboutToWait => {
                window.request_redraw();
            },
            _ => (),
        }
    }).unwrap();
}
