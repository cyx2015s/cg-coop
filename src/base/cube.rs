use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: (f32, f32, f32),
}

implement_vertex!(Vertex, position);
// 右手系顶点定义
pub const VERTICES: [Vertex; 25] = [
    Vertex {
        position: (0.0, 0.0, 0.0),
    }, // dummy vector because in the original model indices
    // 定义前面， 朝向z轴正方向
    Vertex {
        position: (-1.0, -1.0, 1.0),
    },
    Vertex {
        position: (1.0, -1.0, 1.0),
    },
    Vertex {
        position: (1.0, 1.0, 1.0),
    },
    Vertex {
        position: (-1.0, 1.0, 1.0),
    },
    // 定义后面， 朝向z轴负方向
    Vertex {
        position: (-1.0, -1.0, -1.0),
    },
    Vertex {
        position: (1.0, -1.0, -1.0),
    },
    Vertex {
        position: (1.0, 1.0, -1.0),
    },
    Vertex {
        position: (-1.0, 1.0, -1.0),
    },
    // 定义顶面，chao 向y轴正方向
    Vertex {
        position: (-1.0, 1.0, 1.0),
    },
    Vertex {
        position: (1.0, 1.0, 1.0),
    },
    Vertex {
        position: (1.0, 1.0, -1.0),
    },
    Vertex {
        position: (-1.0, 1.0, -1.0),
    },
    // 定义底面，朝向y轴负方向
    Vertex {
        position: (-1.0, -1.0, 1.0),
    },
    Vertex {
        position: (1.0, -1.0, 1.0),
    },
    Vertex {
        position: (1.0, -1.0, -1.0),
    },
    Vertex {
        position: (-1.0, -1.0, -1.0),
    },
    // 定义左面，朝向x轴负方向
    Vertex {
        position: (-1.0, -1.0, 1.0),
    },
    Vertex {
        position: (-1.0, 1.0, 1.0),
    },
    Vertex {
        position: (-1.0, 1.0, -1.0),
    },
    Vertex {
        position: (-1.0, -1.0, -1.0),
    },
    // 定义右面，朝向x轴正方向
    Vertex {
        position: (1.0, -1.0, 1.0),
    },
    Vertex {
        position: (1.0, 1.0, 1.0),
    },
    Vertex {
        position: (1.0, 1.0, -1.0),
    },
    Vertex {
        position: (1.0, -1.0, -1.0),
    },
];

#[derive(Copy, Clone)]
pub struct Normal {
    normal: (f32, f32, f32),
}

implement_vertex!(Normal, normal);

pub const NORMALS: [Normal; 25] = [
    // dummy normal because in the original model indices
    Normal {
        normal: (0.0, 0.0, 0.0),
    },
    // 前面
    Normal {
        normal: (0.0, 0.0, 1.0),
    },
    Normal {
        normal: (0.0, 0.0, 1.0),
    },
    Normal {
        normal: (0.0, 0.0, 1.0),
    },
    Normal {
        normal: (0.0, 0.0, 1.0),
    },
    // 后面
    Normal {
        normal: (0.0, 0.0, -1.0),
    },
    Normal {
        normal: (0.0, 0.0, -1.0),
    },
    Normal {
        normal: (0.0, 0.0, -1.0),
    },
    Normal {
        normal: (0.0, 0.0, -1.0),
    },
    // 顶面
    Normal {
        normal: (0.0, 1.0, 0.0),
    },
    Normal {
        normal: (0.0, 1.0, 0.0),
    },
    Normal {
        normal: (0.0, 1.0, 0.0),
    },
    Normal {
        normal: (0.0, 1.0, 0.0),
    },
    // 底面
    Normal {
        normal: (0.0, -1.0, 0.0),
    },
    Normal {
        normal: (0.0, -1.0, 0.0),
    },
    Normal {
        normal: (0.0, -1.0, 0.0),
    },
    Normal {
        normal: (0.0, -1.0, 0.0),
    },
    // 左面
    Normal {
        normal: (-1.0, 0.0, 0.0),
    },
    Normal {
        normal: (-1.0, 0.0, 0.0),
    },
    Normal {
        normal: (-1.0, 0.0, 0.0),
    },
    Normal {
        normal: (-1.0, 0.0, 0.0),
    },
    // 右面
    Normal {
        normal: (1.0, 0.0, 0.0),
    },
    Normal {
        normal: (1.0, 0.0, 0.0),
    },
    Normal {
        normal: (1.0, 0.0, 0.0),
    },
    Normal {
        normal: (1.0, 0.0, 0.0),
    },
];

pub const INDICES: [u16; 36] = [
    // 前面
    1, 2, 3, 1, 3, 4, // 后面
    5, 6, 7, 5, 7, 8, // 顶面
    9, 10, 11, 9, 11, 12, // 底面
    13, 14, 15, 13, 15, 16, // 左面
    17, 18, 19, 17, 19, 20, // 右面
    21, 22, 23, 21, 23, 24,
];
