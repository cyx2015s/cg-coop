use glium::winit::keyboard::{KeyCode, PhysicalKey};
use glium::*;
pub mod shader;
fn main() {
    // 定义着色器的路径
    let vertex_path = "assets/shaders/vertex.vert";
    let fragment_path = "assets/shaders/fragment.frag";

    // 启动
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .unwrap();
    let (_window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Project")
        .build(&event_loop);

    // 定义顶点的数组格式
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
    }

    // 自动实现一些和顶点有关的方法
    implement_vertex!(Vertex, position);

    // 形状是由顶点组成的数组
    let shape = vec![
        Vertex {
            position: [-0.5, -0.5],
        },
        Vertex {
            position: [-0.5, 0.5],
        },
        Vertex {
            position: [0.5, 0.5],
        },
        Vertex {
            position: [0.5, -0.5],
        },
    ];

    // 索引
    let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

    // 创建顶点缓冲区
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    // 创建索引缓冲区
    let index_buffer = glium::index::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &indices,
    )
    .unwrap();

    // 通过调用 shader库中的 create_shader 函数来创建着色器程序
    let program = shader::create_shader(&display, vertex_path, fragment_path);

    #[allow(deprecated)]
    event_loop
        .run(move |ev, window_target| match ev {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => {
                    // 请求关闭
                    window_target.exit();
                }
                glium::winit::event::WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => {
                    if key_event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                        window_target.exit();
                    } else if key_event.physical_key == PhysicalKey::Code(KeyCode::KeyW) {
                        println!("key-W pressed");
                    } else if key_event.physical_key == PhysicalKey::Code(KeyCode::KeyA) {
                        println!("key-A pressed");
                    } else if key_event.physical_key == PhysicalKey::Code(KeyCode::KeyS) {
                        println!("key-S pressed");
                    } else if key_event.physical_key == PhysicalKey::Code(KeyCode::KeyD) {
                        println!("key-D pressed");
                    }
                }
                glium::winit::event::WindowEvent::RedrawRequested => {
                    // 请求重绘
                    let mut target = display.draw();
                    target.clear_color(0.0, 0.0, 1.0, 1.0);
                    target
                        .draw(
                            &vertex_buffer,
                            &index_buffer,
                            &program,
                            &glium::uniforms::EmptyUniforms,
                            &Default::default(),
                        )
                        .unwrap();
                    target.finish().unwrap();
                }
                _ => (),
            },
            _ => (),
        })
        .unwrap();
}
