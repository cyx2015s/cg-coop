use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
    pub normal: [f32; 3],
}
implement_vertex!(Vertex, position, tex_coord, normal);

#[derive(Copy, Clone)]
pub struct Triangle <'a> {
    pub v: [u16; 3],
    pub vertices: &'a [[f32; 3]],
}

