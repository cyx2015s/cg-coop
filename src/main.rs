use std::time::Instant;

use cg_coop::render::SceneRenderer;
use cg_coop::scene::{World, world};
use cg_coop::ui::{UIBuild, UIHandle};

use glium::winit::event::{DeviceEvent, Event, WindowEvent};
use glium::*;
use imgui::Condition;
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
    // scene.init_aimlab_scene(&display);
    scene.init_house_scene(&display);

    #[allow(deprecated)]
    event_loop.run(move |ev, window_target| {
        global_ctx.handle_event(&ev, &window);
        match ev {
            Event::DeviceEvent { device_id, event } => {
                if let DeviceEvent::MouseMotion { delta } = event {
                    scene.handle_mouse_move(delta, &window);
                }
            }
            Event::WindowEvent { event, .. } => match event {
                // 添加鼠标点击处理
                WindowEvent::MouseInput { state, button, .. } => {
                    use glium::winit::event::{ElementState, MouseButton};
                    if state == ElementState::Pressed && button == MouseButton::Left {
                        // 只有当鼠标被锁定(在玩游戏)时才触发射击，避免点UI时误触
                        if scene.mouse_state.is_locked() {
                            scene.handle_shoot();
                        }
                    }
                },
                WindowEvent::RedrawRequested => {
                    global_ctx.update_time();
                    let mut target = display.draw();
                    target.clear_color_and_depth((0.05, 0.05, 0.1, 1.0), 1.0);
                    let ui = global_ctx.ui_ctx.frame();
                    { 
                        // 获取屏幕尺寸
                        let [width, height] = ui.io().display_size;
                        let center_x = width / 2.0;
                        let center_y = height / 2.0;
                        
                        // 获取画笔
                        let draw_list = ui.get_background_draw_list();
                        
                        // 绘制准星
                        let crosshair_size = 10.0;
                        let crosshair_color = [0.0, 1.0, 0.0, 0.8]; 
                        let thickness = 2.0;

                        draw_list.add_line(
                            [center_x - crosshair_size, center_y],
                            [center_x + crosshair_size, center_y],
                            crosshair_color,
                        ).thickness(thickness).build();

                        draw_list.add_line(
                            [center_x, center_y - crosshair_size],
                            [center_x, center_y + crosshair_size],
                            crosshair_color,
                        ).thickness(thickness).build();
                        
                    } 
                    let _cn_font = ui.push_font(global_ctx.cn_font);

                    ui.show_demo_window(&mut true);

                    

                    scene.build_ui(ui);

                    target.clear_color_and_depth((0.05, 0.05, 0.1, 1.0), 1.0);



                    _cn_font.pop();
                    let now = Instant::now();
                    let mut dt = (now - last_frame).as_secs_f32();
                    if dt > 1.0 { dt = 0.0; }
                    
                    last_frame = now;
                    scene.handle_ui_input(ui, &display);
                    scene.step(dt);
                    scene_renderer.render(&display, &mut scene,&mut target);

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
