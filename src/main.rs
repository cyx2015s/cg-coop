use cg_coop::base::cube;
use cg_coop::base::keystate::InputState;
use cg_coop::base::light::{AmbientLight, DirectionalLight, PointLight, SpotLight};
use cg_coop::base::material::*;
use cg_coop::base::mouse;
use cg_coop::camera;
use cg_coop::shader;
use glium::winit::event::{DeviceEvent, ElementState, Event, WindowEvent};
use glium::winit::keyboard::KeyCode;
use glium::*;
use imgui::Condition;
use imgui::FontConfig;
use imgui::FontGlyphRanges;
use imgui::FontSource;
use std::time::Instant;

fn _print_type<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}
fn main() {
    // 定义灯光和材质
    let mut lambertian = Lambertian::new([1.0, 0.1, 0.1], [1.0, 0.1, 0.1]);
    let mut ambient_light = AmbientLight::new(0.2);
    let mut directional_light =
        DirectionalLight::new([0.0, 0.0, 1.0], [0.0, 1.0, -1.0], 5.0, [1.0, 1.0, 1.0]);
    let mut point_light = PointLight {
        position: [2.0, 2.0, 2.0],
        intensity: 0.0,
        color: [1.0, 1.0, 1.0],
        kc: 1.0,
        kl: 0.09,
        kq: 0.032,
    };
    let mut spot_light = SpotLight {
        position: [0.0, 5.0, 0.0],
        direction: [0.0, -1.0, 0.0],
        intensity: 10.0,
        color: [1.0, 1.0, 1.0],
        angle: 30.0,
        kc: 1.0,
        kl: 0.09,
        kq: 10.2,
    };

    // 定义时间戳
    let mut input_state = InputState::new();
    let mut last_frame_time = Instant::now();
    let ui_last_frame_time = Instant::now();
    // 定义着色器的路径
    let vertex_path = "assets/shaders/lambert.vert";
    let fragment_path = "assets/shaders/lambert.frag";
    let global_ctx = cg_coop::ctx::GlobalContext {
        ui_ctx: imgui::Context::create(),
    };

    // 启动
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Project")
        .build(&event_loop);

    let mut ui_ctx = global_ctx.ui_ctx;
    let mut ui_renderer = imgui_glium_renderer::Renderer::new(&mut ui_ctx, &display).unwrap();
    let mut ui_platform = imgui_winit_support::WinitPlatform::new(&mut ui_ctx);
    let mut ui_last_frame_time = Instant::now();
    // Font
    let cn_font = ui_ctx.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../assets/fonts/font.ttf"),
        size_pixels: 32.0,
        config: Some(FontConfig {
            glyph_ranges: FontGlyphRanges::chinese_simplified_common(),
            ..Default::default()
        }),
    }]);
    ui_renderer
        .reload_font_texture(&mut ui_ctx)
        .expect("字体加载失败");

    ui_platform.attach_window(
        ui_ctx.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Locked(1.0),
    );

    let positions = glium::VertexBuffer::new(&display, &cube::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &cube::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &cube::INDICES,
    )
    .unwrap();
    let (width, height) = display.get_framebuffer_dimensions();
    let aspect_ratio = width as f32 / height as f32;

    // 初始化
    let mut camera = camera::Camera::new(aspect_ratio);
    camera.transform.position = [0.0, 0.0, 5.0].into();
    // 初始化鼠标
    let mut mouse_state = mouse::MouseState::new();
    // 定义顶点的数组格式
    // #[derive(Copy, Clone)]
    // struct Vertex {
    //     position: [f32; 2],
    // }

    // // 自动实现一些和顶点有关的方法
    // implement_vertex!(Vertex, position);

    // // 形状是由顶点组成的数组
    // let shape = vec![
    //     Vertex {
    //         position: [-0.5, -0.5],
    //     },
    //     Vertex {
    //         position: [-0.5, 0.5],
    //     },
    //     Vertex {
    //         position: [0.5, 0.5],
    //     },
    //     Vertex {
    //         position: [0.5, -0.5],
    //     },
    // ];

    // // 索引
    // let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

    // 创建顶点缓冲区
    // let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    // // 创建索引缓冲区
    // let index_buffer = glium::index::IndexBuffer::new(
    //     &display,
    //     glium::index::PrimitiveType::TrianglesList,
    //     &indices,
    // )
    // .unwrap();

    // 通过调用 shader库中的 create_shader 函数来创建着色器程序
    let program = shader::create_shader(&display, vertex_path, fragment_path);
    #[allow(deprecated)]
    event_loop
        .run(move |ev, window_target| {
            // println!("{:?}", &ev);
            ui_platform.handle_event(ui_ctx.io_mut(), &window, &ev);
            match ev {
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
                        if mouse_state.is_locked {
                            let (dx, dy) = delta;
                            mouse_state.delta = (dx as f32, dy as f32);
                            // 应用相机旋转
                            camera.rotate(-mouse_state.delta.0 * mouse_state.sensitivity, -mouse_state.delta.1 * mouse_state.sensitivity);
                           window.request_redraw();
                        }
                    },
                    DeviceEvent::MouseWheel { delta } => {
                        let scroll = match delta {
                            glium::winit::event::MouseScrollDelta::LineDelta(_, y) => y * 50.0,  // 每行当 50 像素
                            glium::winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                        };

                        // exp 缩放 —— 最推荐的相机缩放方式
                        let zoom_sensitivity = 0.005; // 你可以调高调低

                        camera.fovy *= f32::exp(-scroll * zoom_sensitivity);

                        // 限制 FOV（单位：弧度）
                        camera.fovy = camera.fovy.clamp(0.05, 1.5);
                        window.request_redraw();
                    },

                    _ => {}
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => {
                    match key_event.state {
                        ElementState::Pressed => {
                            input_state.set_key_pressed(key_event.physical_key);
                            // 立即响应的按键（如退出）
                            if key_event.physical_key == KeyCode::Escape {
                                window_target.exit();
                            }
                        }
                        ElementState::Released => {
                            input_state.set_key_released(key_event.physical_key);
                            // 只在释放时触发的按键
                            if key_event.physical_key == KeyCode::KeyV {
                                if camera.move_state == camera::MoveState::Free {
                                    camera.move_state = camera::MoveState::Locked;
                                } else {
                                    camera.move_state = camera::MoveState::Free;
                                }
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
                    // 请求重绘
                    let mut target = display.draw();
                    ui_ctx.io_mut().update_delta_time(Instant::now() - ui_last_frame_time);
                    if camera.move_state == camera::MoveState::PanObit {
                        let current_time = Instant::now();
                        let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
                        last_frame_time = current_time;
                        camera.update_pan_obit(delta_time);
                    }
                    ui_last_frame_time = Instant::now();
                    let ui = ui_ctx.frame();
                    let _cn_font = ui.push_font(cn_font);
                    ui.show_demo_window(&mut true);
                    ui.window("操作说明")
                        .size([300.0, 100.0], Condition::FirstUseEver)
                        .build(|| {
                            ui.text_wrapped("按B键漫游\n按P键截图\n按WS在摄像头方向前后移动\n按AD左右移动\n按Space/Ctrl上升下降");     
                        });
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
                        ui.color_edit3("材质 ka", &mut lambertian.ka);
                        ui.color_edit3("材质 kd", &mut lambertian.kd);
                    });
                    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
                    let model = [
                        [0.5, 0.0, 0.0, 0.0],
                        [0.0, 0.5, 0.0, 0.0],
                        [0.0, 0.0, 0.5, 0.0],
                        [0.0, 0.0, 2.0, 1.0f32]
                    ];
                    let view = camera.get_view_matrix();
                    let perspective = camera.get_projection_matrix();
                    let params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            .. Default::default()
                        },
                        //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                        .. Default::default()
                    };
                    let light = [-1.0, 0.4, 0.9f32];
                    target.draw((&positions, &normals), &indices, &program,
                                &uniform! { model: model, view: view, perspective: perspective,
                                material: lambertian.get_mat3_data(),
                                ambient_light: ambient_light.get_mat4_data(),
                                directional_light: directional_light.get_mat4_data(),
                                point_light: point_light.get_mat4_data(),
                                spot_light: spot_light.get_mat4_data(),
                                },
                                &params).unwrap();
                    _cn_font.pop();
                    let draw_data = ui_ctx.render();
                    if draw_data.draw_lists_count() > 0 {
                        ui_renderer.render(&mut target, draw_data).unwrap();
                    }
                    target.finish().unwrap();
                }
                winit::event::WindowEvent::CloseRequested => {
                    window_target.exit();
                }
                _ => (),
            },
            Event::AboutToWait => {
                let current_time = Instant::now();
                let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
                last_frame_time = current_time;
                // 使用 delta_time 确保移动速度与帧率无关
                let move_speed = 3.0; // 单位/秒
                let move_distance = move_speed * delta_time;
                let mut moved = false;
                if camera.move_state == camera::MoveState::Free {
                    if input_state.is_keycode_pressed(KeyCode::KeyW) {
                        camera.transform.position += camera.transform.get_forward() * move_distance;
                        moved = true;
                    }
                    if input_state.is_keycode_pressed(KeyCode::KeyS) {
                        camera.transform.position += -camera.transform.get_forward() * move_distance;
                        moved = true;
                    }
                    if input_state.is_keycode_pressed(KeyCode::KeyA) {
                        camera.transform.position += -camera.transform.get_right() * move_distance;
                        moved = true;
                    }
                    if input_state.is_keycode_pressed(KeyCode::KeyD) {
                        camera.transform.position += camera.transform.get_right() * move_distance;
                        moved = true;
                    }
                    if input_state.is_keycode_pressed(KeyCode::ControlLeft) {
                        camera.transform.position += -glam::f32::Vec3::Y * move_distance;
                        moved = true;
                    }
                    if input_state.is_keycode_pressed(KeyCode::Space)  {
                        camera.transform.position += glam::f32::Vec3::Y * move_distance;
                        moved = true;
                    }
                }
                if moved || true { // 总是重绘以保持流畅
                    window.request_redraw();
                }
            },
            _ => (),
        }})
        .unwrap();
}
