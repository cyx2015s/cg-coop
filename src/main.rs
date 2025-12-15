mod scene;

use cg_coop::base::keystate::InputState;
use cg_coop::base::light::{Light, LightBlock, AmbientLight, DirectionalLight, PointLight, SpotLight};
use cg_coop::base::material;
use cg_coop::base::mouse;
use cg_coop::camera;
use cg_coop::shader;
use glium::winit::event::{DeviceEvent, ElementState, Event, WindowEvent};
use glium::winit::keyboard::KeyCode;
use glium::*;
use imgui::{Condition, FontConfig, FontGlyphRanges, FontSource, Drag}; // 引入 Drag
use std::time::Instant;
use cg_coop::shape::mesh::{AsMesh, Mesh};

use scene::{Scene, GameObject, ShapeKind};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}
implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
struct Normal {
    normal: [f32; 3],
}
implement_vertex!(Normal, normal);

fn main() {
    // 初始化资源路径
    let phong_vertex_path = "assets/shaders/Phong.vert";
    let phong_fragment_path = "assets/shaders/Phong.frag";
    
    // 默认材质
    let default_mat = material::Phong::new([1.0, 0.5, 0.31], [1.0, 0.5, 0.31], [0.5, 0.5, 0.5], 32.0).to_Material();

    // 灯光初始化
    let mut ambient_light = AmbientLight::new(0.2);
    let mut directional_light = DirectionalLight::new([0.0, 0.0, 1.0], [0.0, 1.0, -1.0], 5.0, [1.0, 1.0, 1.0]);
    let mut point_light = PointLight {
        position: [2.0, 2.0, 2.0], intensity: 0.0, color: [1.0, 1.0, 1.0], kc: 1.0, kl: 0.09, kq: 0.032,
    };
    let mut spot_light = SpotLight {
        position: [0.0, 5.0, 0.0], direction: [0.0, -1.0, 0.0], intensity: 10.0, color: [1.0, 1.0, 1.0],
        angle: 30.0, kc: 1.0, kl: 0.09, kq: 10.2,
    };
    // 材质变量
    let mut lambertian = material::Lambertian::new([1.0, 0.1, 0.1], [1.0, 0.1, 0.1]); // 仅用于UI演示

    // 系统初始化
    let mut input_state = InputState::new();
    let mut last_frame_time = Instant::now();
    let global_ctx = cg_coop::ctx::GlobalContext { ui_ctx: imgui::Context::create() };

    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Project - Mini Blender Mode")
        .build(&event_loop);

    // Uniform Buffers
    let light_block = LightBlock { lights: [Light::default(); 32], num_lights: 0 };
    let material_block = material::MaterialBlock { material: material::Material::default() };
    let light_ubo = glium::uniforms::UniformBuffer::new(&display, light_block).unwrap();
    let material_ubo = glium::uniforms::UniformBuffer::new(&display, material_block).unwrap();

    // ImGui 初始化
    let mut ui_ctx = global_ctx.ui_ctx;
    let mut ui_renderer = imgui_glium_renderer::Renderer::new(&mut ui_ctx, &display).unwrap();
    let mut ui_platform = imgui_winit_support::WinitPlatform::new(&mut ui_ctx);
    let mut ui_last_frame_time = Instant::now();

    // 字体加载
    let cn_font = ui_ctx.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../assets/fonts/font.ttf"),
        size_pixels: 20.0, 
        config: Some(FontConfig { glyph_ranges: FontGlyphRanges::chinese_full(), ..Default::default() }),
    }]);
    ui_renderer.reload_font_texture(&mut ui_ctx).expect("字体加载失败");
    ui_platform.attach_window(ui_ctx.io_mut(), &window, imgui_winit_support::HiDpiMode::Locked(1.0));

    // 相机初始化
    let (width, height) = display.get_framebuffer_dimensions();
    let mut camera = camera::Camera::new(width as f32 / height as f32);
    camera.transform.position = [0.0, 2.0, 5.0].into();
    camera.transform.look_at([0.0, 0.0, 0.0].into(), [0.0, 1.0, 0.0].into());
    let mut mouse_state = mouse::MouseState::new();

    let phong_program = shader::create_shader(&display, phong_vertex_path, phong_fragment_path);

    // 场景初始化
    let mut scene = Scene::new();

    // 默认添加地板
    let mut floor = GameObject::new("Floor", ShapeKind::Cube{ width: 10.0, height: 0.1, depth: 10.0 }, default_mat);
    floor.transform.position.y = -1.0;
    scene.add_object(floor);

    // 默认添加球体
    let sphere = GameObject::new("Sphere", ShapeKind::Sphere{ radius: 0.8, sectors: 32 }, default_mat);
    scene.add_object(sphere);

    // 主循环
    #[allow(deprecated)]
    event_loop.run(move |ev, window_target| {
        ui_platform.handle_event(ui_ctx.io_mut(), &window, &ev);
        match ev {
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
                         if mouse_state.is_locked {
                            let (dx, dy) = delta;
                            mouse_state.delta = (dx as f32, dy as f32);
                            camera.rotate(-mouse_state.delta.0 * mouse_state.sensitivity, -mouse_state.delta.1 * mouse_state.sensitivity);
                            window.request_redraw();
                        }
                    },
                    DeviceEvent::MouseWheel { delta } => {
                         let scroll = match delta {
                            glium::winit::event::MouseScrollDelta::LineDelta(_, y) => y * 50.0,
                            glium::winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                        };
                        camera.fovy *= f32::exp(-scroll * 0.005);
                        camera.fovy = camera.fovy.clamp(0.05, 1.5);
                        window.request_redraw();
                    },
                    _ => {}
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                     match key_event.state {
                        ElementState::Pressed => {
                            input_state.set_key_pressed(key_event.physical_key);
                            if key_event.physical_key == KeyCode::Escape { window_target.exit(); }
                        }
                        ElementState::Released => {
                            input_state.set_key_released(key_event.physical_key);
                            if key_event.physical_key == KeyCode::KeyV {
                                camera.move_state = if camera.move_state == camera::MoveState::Free { camera::MoveState::Locked } else { camera::MoveState::Free };
                                mouse_state.toggle_lock(&window);
                            }
                            if key_event.physical_key == KeyCode::KeyB {
                                if camera.move_state == camera::MoveState::PanObit {
                                    camera.stop_pan_obit();
                                } else {
                                    camera.start_pan_obit(0.0, 5.0, [0.5, 0.5, 0.5]);
                                    camera.pan_obit_speed = 10.0;
                                }
                            }
                            if key_event.physical_key == KeyCode::KeyR {
                                camera.fovy = 3.141592 / 2.0;
                            }
                            if key_event.physical_key == KeyCode::KeyP {
                                let image: glium::texture::RawImage2d<'_, u8> = display.read_front_buffer().unwrap();
                                let image = image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
                                let image = image::DynamicImage::ImageRgba8(image).flipv();
                                image.save("screenshot.png").unwrap();
                            }
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    let mut target = display.draw();
                    target.clear_color_and_depth((0.05, 0.05, 0.1, 1.0), 1.0); 

                    ui_ctx.io_mut().update_delta_time(Instant::now() - ui_last_frame_time);
                    
                    // 更新环绕相机逻辑
                    if camera.move_state == camera::MoveState::PanObit {
                        let current_time = Instant::now();
                        let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
                        camera.update_pan_obit(delta_time);
                    }

                    ui_last_frame_time = Instant::now();
                    
                    let ui = ui_ctx.frame();
                    let _cn_font = ui.push_font(cn_font);


                    ui.show_demo_window(&mut true); 

                    // 操作说明

                    ui.window("操作说明")
                        .size([300.0, 100.0], Condition::FirstUseEver)
                        .build(|| {
                            ui.text_wrapped("按V键漫游\n按B键环绕\n滚轮放大缩小视角\n按R恢复视角\n按P键截图\n按WS在摄像头方向前后移动\n按AD左右移动\n按Space/Ctrl上升下降");     
                        });

                    // 灯光与材质测试
                    ui.window("灯光与材质测试").size([400.0, 200.0], Condition::FirstUseEver).build(|| {
                        ui.slider("环境光强度", 0.0, 1.0, &mut ambient_light.intensity);
                        ui.slider("平行光强度", 0.0, 5.0, &mut directional_light.intensity);
                        ui.slider("点光源强度", 0.0, 50.0, &mut point_light.intensity);
                        ui.slider("聚光灯强度", 0.0, 100.0, &mut spot_light.intensity);
                        ui.separator();
                        ui.slider_config("平行光方向", -3.0, 3.0).build_array(&mut directional_light.direction);
                        ui.color_edit3("平行光颜色", &mut directional_light.color);
                        ui.separator();
                        ui.slider_config("点光源位置", -10.0, 10.0).build_array(&mut point_light.position);
                        ui.color_edit3("点光源颜色", &mut point_light.color);
                        ui.separator();
                        ui.text("全局材质预览");
                        ui.color_edit3("材质 ka", &mut lambertian.ka);
                        ui.color_edit3("材质 kd", &mut lambertian.kd);
                    });

                    // 场景列表
                    ui.window("场景列表 (Scene List)").size([200.0, 400.0], Condition::FirstUseEver).position([20.0, 150.0], Condition::FirstUseEver).build(|| {
                        ui.text("基础形状:");
                        if ui.button("立方体") {
                            scene.add_object(GameObject::new("Cube", ShapeKind::Cube{width:1.0, height:1.0, depth:1.0}, default_mat));
                        }
                        if ui.button("球体") {
                            scene.add_object(GameObject::new("Sphere", ShapeKind::Sphere{radius:0.5, sectors:32}, default_mat));
                        }
                        
                        ui.text("柱体/台体:");
                        // 圆柱：顶底半径相等
                        if ui.button("圆柱") {
                            scene.add_object(GameObject::new("Cylinder", ShapeKind::Cylinder{top_radius:0.5, bottom_radius:0.5, height:1.0, sectors:32}, default_mat));
                        }
                        // 棱台：顶底半径不等
                        if ui.button("棱台/圆台") {
                            scene.add_object(GameObject::new("Frustum", ShapeKind::Cylinder{top_radius:0.3, bottom_radius:0.8, height:1.0, sectors:32}, default_mat));
                        }
                        // 圆锥
                        if ui.button("圆锥") {
                            scene.add_object(GameObject::new("Cone", ShapeKind::Cone{radius:0.5, height:1.0, sectors:32}, default_mat));
                        }

                        ui.text("其他:");
                        if ui.button("导入模型") {
                            if let Ok(mesh) = Mesh::load_obj("output.obj") {
                                let mut obj = GameObject::new("Imported", ShapeKind::Imported, default_mat);
                                obj.mesh = mesh;
                                scene.add_object(obj);
                            }
                        }
                        
                        ui.separator();
                        ui.text("场景物体:");
                        
                        // 遍历显示所有物体
                        for (i, obj) in scene.objects.iter().enumerate() {
                            let is_selected = scene.selected_index == Some(i);
                            if ui.selectable_config(&format!("{}: {}", i, obj.name)).selected(is_selected).build() {
                                scene.selected_index = Some(i);
                            }
                        }
                    });

                    // 属性面板
                    if let Some(obj) = scene.get_selected_mut() {
                        ui.window("属性面板 (Inspector)").size([250.0, 400.0], Condition::FirstUseEver).position([240.0, 150.0], Condition::FirstUseEver).build(|| {
                            ui.text_colored([0.0, 1.0, 0.0, 1.0], &format!("当前选中: {}", obj.name));
                            ui.separator();

                            ui.text("变换 (Transform)");
                            let mut pos = obj.transform.position.to_array();
                            if Drag::new("位置").speed(0.1).build_array(ui, &mut pos) {
                                obj.transform.position = pos.into();
                            }
                            
                            let mut scale = obj.transform.scale.to_array();
                            if Drag::new("缩放").speed(0.01).build_array(ui, &mut scale) {
                                obj.transform.scale = scale.into();
                            }

                            if ui.button("重置旋转") {
                                obj.transform.rotation = glam::f32::Quat::IDENTITY;
                            }

                            ui.separator();
                            ui.text("形状参数 (Parameters)");
                            
                            let mut need_regen = false;
                            match &mut obj.kind {
                                ShapeKind::Cube { width, height, depth } => {
                                    if Drag::new("宽").speed(0.1).build(ui, width) { need_regen = true; }
                                    if Drag::new("高").speed(0.1).build(ui, height) { need_regen = true; }
                                    if Drag::new("深").speed(0.1).build(ui, depth) { need_regen = true; }
                                },
                                ShapeKind::Sphere { radius, sectors } => {
                                    if Drag::new("半径").speed(0.05).build(ui, radius) { need_regen = true; }
                                    let mut s = *sectors as i32;
                                    if ui.slider("精度", 3, 64, &mut s) { *sectors = s as u16; need_regen = true; }
                                },
                                // Cylinder 现在处理 圆柱、棱柱、棱台
                                ShapeKind::Cylinder { top_radius, bottom_radius, height, sectors } => {
                                    ui.text("几何尺寸:");
                                    if Drag::new("顶半径").speed(0.05).build(ui, top_radius) { need_regen = true; }
                                    if Drag::new("底半径").speed(0.05).build(ui, bottom_radius) { need_regen = true; }
                                    if Drag::new("高度").speed(0.1).build(ui, height) { need_regen = true; }
                                    
                                    ui.separator();
                                    ui.text("边数控制 (棱柱调节这里):");
                                    // 棱柱就是边数很少的圆柱
                                    let mut s = *sectors as i32;
                                    if ui.slider("精度/边数", 3, 64, &mut s) { 
                                        *sectors = s as u16; 
                                        need_regen = true; 
                                    }
                                    if *sectors < 5 {
                                        ui.text_colored([1.0, 1.0, 0.0, 1.0], "提示: 低精度即为棱柱/棱台");
                                    }
                                },
                                // 圆锥面板
                                ShapeKind::Cone { radius, height, sectors } => {
                                    if Drag::new("底半径").speed(0.05).build(ui, radius) { need_regen = true; }
                                    if Drag::new("高度").speed(0.1).build(ui, height) { need_regen = true; }
                                    let mut s = *sectors as i32;
                                    if ui.slider("精度", 3, 64, &mut s) { *sectors = s as u16; need_regen = true; }
                                },
                                _ => {}
                            }

                            if need_regen {
                                obj.regenerate_mesh();
                            }
                            
                            ui.separator();
                            ui.checkbox("显示/隐藏", &mut obj.visible);
                            if ui.button("保存当前模型") {
                                let _ = obj.mesh.save_obj("output.obj");
                            }
                        });
                    }

                    // 渲染循环 (遍历场景) 
                    let mut l_block = LightBlock { lights: [Light::default(); 32], num_lights: 0 };
                    l_block.lights[0] = ambient_light.to_Light(); l_block.num_lights += 1;
                    l_block.lights[1] = directional_light.to_Light(); l_block.num_lights += 1;
                    l_block.lights[2] = point_light.to_Light(); l_block.num_lights += 1;
                    l_block.lights[3] = spot_light.to_Light(); l_block.num_lights += 1; // 加上聚光灯
                    light_ubo.write(&l_block);

                    let view = camera.get_view_matrix();
                    let perspective = camera.get_projection_matrix();
                    let viewPos = camera.get_position();

                    let params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            .. Default::default()
                        },
                        .. Default::default()
                    };

                    for obj in &scene.objects {
                        if !obj.visible { continue; }
                        let model = obj.transform.get_matrix().to_cols_array_2d();
                        let m_block = material::MaterialBlock { material: obj.material };
                        material_ubo.write(&m_block);

                        let vertex_data: Vec<Vertex> = obj.mesh.vertices.iter().map(|v| Vertex { position: *v }).collect();
                        let normal_data: Vec<Normal> = obj.mesh.normals.iter().map(|n| Normal { normal: *n }).collect();
                        
                        if vertex_data.is_empty() { continue; }

                        let positions = glium::VertexBuffer::new(&display, &vertex_data).unwrap();
                        let normals = glium::VertexBuffer::new(&display, &normal_data).unwrap();
                        let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &obj.mesh.indices).unwrap();

                        target.draw((&positions, &normals), &indices, &phong_program,
                            &uniform! { 
                                model: model, 
                                view: view, 
                                perspective: perspective,
                                viewPos: viewPos,
                                Material_Block: &material_ubo,
                                Light_Block: &light_ubo,
                            },
                            &params).unwrap();
                    }

                    _cn_font.pop();
                    let draw_data = ui_ctx.render();
                    if draw_data.draw_lists_count() > 0 {
                        ui_renderer.render(&mut target, draw_data).unwrap();
                    }
                    target.finish().unwrap();
                }
                winit::event::WindowEvent::CloseRequested => { window_target.exit(); }
                _ => (),
            },
            Event::AboutToWait => {
                let current_time = Instant::now();
                let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
                last_frame_time = current_time;
                let move_speed = 5.0 * delta_time;
                
                if camera.move_state == camera::MoveState::Free {
                    if input_state.is_keycode_pressed(KeyCode::KeyW) { camera.transform.position += camera.transform.get_forward() * move_speed; }
                    if input_state.is_keycode_pressed(KeyCode::KeyS) { camera.transform.position -= camera.transform.get_forward() * move_speed; }
                    if input_state.is_keycode_pressed(KeyCode::KeyA) { camera.transform.position -= camera.transform.get_right() * move_speed; }
                    if input_state.is_keycode_pressed(KeyCode::KeyD) { camera.transform.position += camera.transform.get_right() * move_speed; }
                    if input_state.is_keycode_pressed(KeyCode::Space) { camera.transform.position += glam::f32::Vec3::Y * move_speed; }
                    if input_state.is_keycode_pressed(KeyCode::ControlLeft) { camera.transform.position -= glam::f32::Vec3::Y * move_speed; }
                }
                window.request_redraw();
            },
            _ => (),
        }
    }).unwrap();
}