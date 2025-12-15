use cg_coop::base::material::{self, Material};
use cg_coop::base::transform::Transform;
use cg_coop::shape::mesh::{Mesh, AsMesh};
use cg_coop::shape::{cube::Cube, sphere::Sphere, cylinder::Cylinder, cone::Cone};

#[derive(Clone)]
pub enum ShapeKind {
    Cube { width: f32, height: f32, depth: f32 },
    Sphere { radius: f32, sectors: u16 },
    // 修改：将 radius 改为 top/bottom 两个半径，这样就能表示圆柱、棱台、棱柱
    Cylinder { top_radius: f32, bottom_radius: f32, height: f32, sectors: u16 },
    // 确保圆锥存在
    Cone { radius: f32, height: f32, sectors: u16 },
    Imported, 
}

pub struct GameObject {
    pub name: String,
    pub transform: Transform,
    pub material: Material,
    pub mesh: Mesh,
    pub kind: ShapeKind,
    pub visible: bool,
}

impl GameObject {
    pub fn new(name: &str, kind: ShapeKind, material: Material) -> Self {
        let mut obj = Self {
            name: name.to_string(),
            transform: Transform::default(),
            material,
            mesh: Mesh { vertices: vec![], normals: vec![], indices: vec![] },
            kind,
            visible: true,
        };
        obj.regenerate_mesh();
        obj
    }

    pub fn regenerate_mesh(&mut self) {
        self.mesh = match &self.kind {
            ShapeKind::Cube { width, height, depth } => {
                let s = Cube { width: *width, height: *height, depth: *depth };
                s.as_mesh()
            },
            ShapeKind::Sphere { radius, sectors } => {
                let s = Sphere { radius: *radius, col_divisions: *sectors, row_divisions: *sectors };
                s.as_mesh()
            },
            // 修改：支持不等径圆柱（即棱台）
            ShapeKind::Cylinder { top_radius, bottom_radius, height, sectors } => {
                let s = Cylinder { 
                    bottom_radius: *bottom_radius, 
                    top_radius: *top_radius, 
                    height: *height, 
                    sectors: *sectors 
                };
                s.as_mesh()
            },
            ShapeKind::Cone { radius, height, sectors } => {
                let s = Cone { radius: *radius, height: *height, sectors: *sectors };
                s.as_mesh()
            },
            ShapeKind::Imported => {
                self.mesh.clone() 
            },
        };
    }
}

pub struct Scene {
    pub objects: Vec<GameObject>,
    pub selected_index: Option<usize>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            selected_index: None,
        }
    }

    pub fn add_object(&mut self, obj: GameObject) {
        self.objects.push(obj);
        self.selected_index = Some(self.objects.len() - 1);
    }
    
    pub fn get_selected_mut(&mut self) -> Option<&mut GameObject> {
        if let Some(idx) = self.selected_index {
            if idx < self.objects.len() {
                return Some(&mut self.objects[idx]);
            }
        }
        None
    }
}