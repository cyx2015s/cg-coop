use glium::*;
use std::fs;
use std::path::Path;
pub fn create_shader<P: AsRef<Path>>(
    display: &impl glium::backend::Facade,
    vertex_path: P,
    fragment_path: P,
) -> Program {
    let vertex_shader_src = fs::read_to_string(&vertex_path).unwrap_or_else(|_| panic!("Something went wrong reading the vertex file from {}",
        vertex_path.as_ref().to_string_lossy()));
    let fragment_shader_src = fs::read_to_string(&fragment_path).unwrap_or_else(|_| panic!("Something went wrong reading the fragment file from {}",
        fragment_path.as_ref().to_string_lossy()));
    glium::Program::from_source(display, &vertex_shader_src, &fragment_shader_src, None)
        .unwrap()
}
