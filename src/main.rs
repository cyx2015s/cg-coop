use cg_coop::base::cube;
use cg_coop::base::keystate::InputState;
use cg_coop::base::mouse;
use cg_coop::camera;
use cg_coop::shader;
use glium::winit::keyboard::KeyCode;
use glium::*;
use std::time::Instant;

fn main() {
    // 定义时间戳
    let mut input_state = InputState::new();
    let mut last_frame_time = Instant::now();
    // 定义着色器的路径
    let vertex_path = "assets/shaders/3d_vertex.vert";
    let fragment_path = "assets/shaders/3d_fragment.frag";

    // 启动
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .unwrap();
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Project")
        .build(&event_loop);

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
        .run(move |ev, window_target| match ev {
            glium::winit::event::Event::DeviceEvent { event, .. } => {
                match event {
                    glium::winit::event::DeviceEvent::MouseMotion { delta } => {
                        if mouse_state.is_locked {
                            let (dx, dy) = delta;
                            mouse_state.delta = (dx as f32, dy as f32);
                            // 应用相机旋转
                            camera.rotate(-mouse_state.delta.0 * mouse_state.sensitivity, -mouse_state.delta.1 * mouse_state.sensitivity);
                           _window.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => {
                    match key_event.state {
                        glium::winit::event::ElementState::Pressed => {
                            input_state.set_key_pressed(key_event.physical_key);
                            // 立即响应的按键（如退出）
                            if key_event.physical_key == KeyCode::Escape {
                                window_target.exit();
                            }
                        }
                        glium::winit::event::ElementState::Released => {
                            input_state.set_key_released(key_event.physical_key);
                            // 只在释放时触发的按键
                            if key_event.physical_key == KeyCode::KeyV {
                                mouse_state.toggle_lock(&_window);
                            }
                        }
                    }
                }
                glium::winit::event::WindowEvent::RedrawRequested => {
                    // 请求重绘
                    let mut target = display.draw();
                    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
                    let model = [
                        [0.5, 0.0, 0.0, 0.0],
                        [0.0, 0.5, 0.0, 0.0],
                        [0.0, 0.0, 0.5, 0.0],
                        [0.0, 0.0, 2.0, 1.0f32]
                    ];
                    let view = camera.get_view_matrix();
                    let perspective = camera.get_projection_matrix();
                    let light = [-1.0, 0.4, 0.9f32];
                    let params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            .. Default::default()
                        },
                        //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                        .. Default::default()
                    };
                    target.draw((&positions, &normals), &indices, &program,
                                &uniform! { model: model, view: view, perspective: perspective, u_light: light },
                                &params).unwrap();
                    target.finish().unwrap();
                }
                _ => (),
            },
            glium::winit::event::Event::AboutToWait => {
                            let current_time = Instant::now();
            let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = current_time;
            // 使用 delta_time 确保移动速度与帧率无关
            let move_speed = 3.0; // 单位/秒
            let move_distance = move_speed * delta_time;
            let mut moved = false;
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
            if input_state.is_keycode_pressed(KeyCode::Space) {
                camera.transform.position += glam::f32::Vec3::Y * move_distance;
                moved = true;
            }
            if moved || true { // 总是重绘以保持流畅
                _window.request_redraw();
            }
            },
            _ => (),
        })
        .unwrap();
}
