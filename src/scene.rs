use cg_coop::base::material::{self, Material};
use cg_coop::base::transform::Transform;
use cg_coop::shape::mesh::{Mesh, AsMesh};
use cg_coop::shape::{cube::Cube, sphere::Sphere, cylinder::Cylinder, cone::Cone};
use cg_coop::shape::nurbs::NurbsSurface;

#[derive(Clone)]
pub enum ShapeKind {
    Cube { width: f32, height: f32, depth: f32 },
    Sphere { radius: f32, sectors: u16 },
    Cylinder { top_radius: f32, bottom_radius: f32, height: f32, sectors: u16 },
    Cone { radius: f32, height: f32, sectors: u16 },
    Imported, 
    Nurbs { 
        degree: usize, 
        control_points: Vec<[f32; 3]>, 
        weights: Vec<f32>,
        u_count: usize,
        v_count: usize
    },
}

pub struct GameObject {
    pub name: String,
    pub transform: Transform,
    pub material: Material,
    pub mesh: Mesh,
    pub kind: ShapeKind,
    pub visible: bool,
    pub use_texture: bool,
}

impl GameObject {
    pub fn new(name: &str, kind: ShapeKind, material: Material) -> Self {
        let mut obj = Self {
            name: name.to_string(),
            transform: Transform::default(),
            material,
            mesh: Mesh { vertices: vec![], normals: vec![], tex_coords: vec![], indices: vec![] },
            kind,
            visible: true,
            use_texture: false,
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
            ShapeKind::Nurbs { degree, control_points, weights, u_count, v_count } => {
                 let s = NurbsSurface {
                     control_points: control_points.clone(),
                     weights: weights.clone(),
                     u_count: *u_count,
                     v_count: *v_count,
                     degree: *degree,
                     splits: 32, 
                 };
                 s.as_mesh()
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