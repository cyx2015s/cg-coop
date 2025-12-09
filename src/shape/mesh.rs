pub struct Mesh {
    // Common mesh data fields, no uv, no colors
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// 3 indices per triangle
    pub indices: Vec<u16>,
}

pub trait AsMesh {
    fn as_mesh(&self) -> Mesh;
}
