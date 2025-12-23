use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coord);

#[derive(Copy, Clone)]
pub struct Normal {
    pub normal: [f32; 3],
}
implement_vertex!(Normal, normal);