use glium::*;

fn main() {
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

    // 编译着色器
    let vertex_shader_src = r#"
           #version 140

           in vec2 position;
           out vec2 f_position;
           void main() {
               gl_Position = vec4(position, 0.0, 1.0);
               f_position = position;
           }
       "#;

    let fragment_shader_src = r#"
           #version 140
           in vec2 f_position;
           out vec4 color;

           void main() {
               color = vec4(f_position * 0.5 + 0.5, f_position.x * f_position.y + 0.5, 1.0);
           }
       "#;

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    #[allow(deprecated)]
    event_loop
        .run(move |ev, window_target| match ev {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => {
                    // 请求关闭
                    window_target.exit();
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
