use cg_coop::base::cube;
use cg_coop::base::keystate::InputState;
use cg_coop::base::light::{Light, LightBlock, AmbientLight, DirectionalLight, PointLight, SpotLight};
use cg_coop::base::material;
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
use cg_coop::shape::mesh::AsMesh; 
use cg_coop::shape::{cube::Cube, sphere::Sphere, cylinder::Cylinder, cone::Cone};

fn _print_type<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

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

    // 定义着色器的路径
    let phong_vertex_path = "assets/shaders/Phong.vert";
    let phong_fragment_path = "assets/shaders/Phong.frag";

    let lambert_vertex_path = "assets/shaders/Lambert.vert";
    let lambert_fragment_path = "assets/shaders/Lambert.frag";
    // 定义灯光和材质
    let mut lambertian = material::Lambertian::new([1.0, 0.1, 0.1], [1.0, 0.1, 0.1]);
    let mut phong = material::Phong::new([1.0, 0.03, 0.03], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0], 10.0);
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

    let global_ctx = cg_coop::ctx::GlobalContext {
        ui_ctx: imgui::Context::create(),
    };

    // 启动
    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Project")
        .build(&event_loop);

    let mut light_block = LightBlock { 
        lights: [Light::default(); 32],
        num_lights: 0,
    };

    let mut material_block = material::MaterialBlock {
        material: material::Material::default(),
    };

    let light_ubo = glium::uniforms::UniformBuffer::new(&display, light_block).unwrap();
    let material_ubo = glium::uniforms::UniformBuffer::new(&display, material_block).unwrap();

    let mut ui_ctx = global_ctx.ui_ctx;
    let mut ui_renderer = imgui_glium_renderer::Renderer::new(&mut ui_ctx, &display).unwrap();
    let mut ui_platform = imgui_winit_support::WinitPlatform::new(&mut ui_ctx);
    let mut ui_last_frame_time = Instant::now();
    // Font
    let cn_font = ui_ctx.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../assets/fonts/font.ttf"),
        size_pixels: 32.0,
        config: Some(FontConfig {
            glyph_ranges: FontGlyphRanges::chinese_full(),
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

    // 定义初始状态 
    let mut cy_radius = 1.0f32;      // 底面半径
    let mut cy_top_radius = 0.5f32;  // 顶面半径
    let mut cy_height = 2.0f32;      // 高度
    let mut cy_sectors = 6i32;       // 切分份数

    // 形状选择器
    let mut shape_type = 4; // 默认选中棱台看效果

    // 通过调用 shader库中的 create_shader 函数来创建着色器程序
    let phong_program = shader::create_shader(&display, phong_vertex_path, phong_fragment_path);
    let lambert_program = shader::create_shader(&display, lambert_vertex_path, lambert_fragment_path);
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
                            ui.text_wrapped("按V键漫游\n按B键环绕\n滚轮放大缩小视角\n按R恢复视角\n按P键截图\n按WS在摄像头方向前后移动\n按AD左右移动\n按Space/Ctrl上升下降");     
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
                    ui.window("建模实验室").size([300.0, 350.0], Condition::FirstUseEver).build(|| {
                        ui.text("选择形状 (Shape Select)");
                        ui.radio_button("立方体 (Cube)", &mut shape_type, 0);
                        ui.radio_button("球体 (Sphere)", &mut shape_type, 1);
                        ui.radio_button("圆柱 (Cylinder)", &mut shape_type, 2);
                        ui.radio_button("圆锥 (Cone)", &mut shape_type, 3);
                        ui.radio_button("多面棱台/棱柱 (Frustum/Prism)", &mut shape_type, 4);
                        
                        ui.separator();
                        ui.text("参数调整 (Parameters)");
                        
                        // 根据不同的形状显示不同的滑块
                        match shape_type {
                            1 => { // Sphere
                                ui.slider("半径 (Radius)", 0.1, 5.0, &mut cy_radius);
                                ui.slider("精度 (Sectors)", 3, 64, &mut cy_sectors);
                            },
                            2 => { // Cylinder
                                ui.slider("半径 (Radius)", 0.1, 5.0, &mut cy_radius);
                                ui.slider("高度 (Height)", 0.1, 5.0, &mut cy_height);
                                ui.slider("精度 (Sectors)", 3, 64, &mut cy_sectors);
                            },
                            3 => { // Cone
                                ui.slider("底面半径 (Radius)", 0.1, 5.0, &mut cy_radius);
                                ui.slider("高度 (Height)", 0.1, 5.0, &mut cy_height);
                                ui.slider("精度 (Sectors)", 3, 64, &mut cy_sectors);
                            },
                            4 => { // Frustum / Prism (多面棱台)
                                ui.slider("底面半径 (Bottom R)", 0.1, 5.0, &mut cy_radius);
                                ui.slider("顶面半径 (Top R)", 0.0, 5.0, &mut cy_top_radius); // 可以是0，就是棱锥
                                ui.slider("高度 (Height)", 0.1, 5.0, &mut cy_height);
                                ui.slider("侧面数 (Sides)", 3, 64, &mut cy_sectors); // 限制在3-64
                            },
                            _ => {} // Cube 不需要参数
                        }
                    });
                    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
                    let model = [
                        [0.5, 0.0, 0.0, 0.0],
                        [0.0, 0.5, 0.0, 0.0],
                        [0.0, 0.0, 0.5, 0.0],
                        [0.0, 0.0, 2.0, 1.0f32]
                    ];
                    let view = camera.get_view_matrix();
                    let viewPos = camera.get_position();
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
                    let mut l_block = LightBlock {
                        lights: [Light::default(); 32], // 初始化数组，每个元素都是 Light::default()
                        num_lights: 0,
                    };

                    l_block.lights[l_block.num_lights as usize] = ambient_light.to_Light();
                    l_block.num_lights += 1;
                    l_block.lights[l_block.num_lights as usize] = directional_light.to_Light();
                    l_block.num_lights += 1;
                    l_block.lights[l_block.num_lights as usize] = point_light.to_Light();
                    l_block.num_lights += 1;
                    l_block.lights[l_block.num_lights as usize] = spot_light.to_Light();
                    l_block.num_lights += 1;

                    let mut m_block = material::MaterialBlock { material: phong.to_Material() };

                    light_ubo.write(&l_block);
                    material_ubo.write(&m_block);
                    let light = [-1.0, 0.4, 0.9f32];

                    // 每一帧都根据最新参数生成模型，根据 shape_type 动态决定画什么
                    let mesh_data = match shape_type {
                        0 => { // Cube
                             let s = Cube { width: cy_radius * 2.0, height: cy_radius * 2.0, depth: cy_radius * 2.0 };
                             s.as_mesh()
                        },
                        1 => { // Sphere
                             let s = Sphere { radius: cy_radius, col_divisions: cy_sectors as u16, row_divisions: cy_sectors as u16 };
                             s.as_mesh()
                        },
                        2 => { // Cylinder
                             let s = Cylinder { 
                                 bottom_radius: cy_radius, 
                                 top_radius: cy_radius, 
                                 height: cy_height, 
                                 sectors: cy_sectors as u16 
                             };
                             s.as_mesh()
                        },
                        3 => { // Cone
                             let s = Cone { radius: cy_radius, height: cy_height, sectors: cy_sectors as u16 };
                             s.as_mesh()
                        },
                        4 => { // Frustum (多面棱台)
                             let s = Cylinder { 
                                 bottom_radius: cy_radius, 
                                 top_radius: cy_top_radius, // 使用独立的顶面半径
                                 height: cy_height, 
                                 sectors: cy_sectors as u16 
                             };
                             s.as_mesh()
                        },
                        _ => {
                             let s = Sphere { radius: 1.0, col_divisions: 32, row_divisions: 32 };
                             s.as_mesh()
                        }
                    };

                    // 转换格式 (为了 Glium)
                    let vertex_data: Vec<Vertex> = mesh_data.vertices
                        .iter()
                        .map(|v| Vertex { position: *v })
                        .collect();

                    let normal_data: Vec<Normal> = mesh_data.normals
                        .iter()
                        .map(|n| Normal { normal: *n })
                        .collect();

                    // 传给显卡
                    let positions = glium::VertexBuffer::new(&display, &vertex_data).unwrap();
                    let normals = glium::VertexBuffer::new(&display, &normal_data).unwrap();
                    let indices = glium::IndexBuffer::new(
                        &display,
                        glium::index::PrimitiveType::TrianglesList,
                        &mesh_data.indices,
                    ).unwrap();
                    
                    target.draw((&positions, &normals), &indices, &phong_program,
                                &uniform! { model: model, view: view, perspective: perspective,
                                viewPos: viewPos,
                                Material_Block: &material_ubo,
                                Light_Block: &light_ubo,
                                },
                                &params).unwrap();
                                
                    // target.draw((&positions, &normals), &indices, &lambert_program,
                    //             &uniform! { model: model, view: view, perspective: perspective,
                    //             Material_Block: &material_ubo,
                    //             Light_Block: &light_ubo,
                    //             },
                    //             &params).unwrap();
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
