use super::camera::{Camera, MouseState};
use super::light::{AmbientLight, DirectionalLight, Light, PointLight, SpotLight};

use crate::core::material::{Material, Phong};
use crate::core::math::aabb::AABB;
use crate::core::math::transform::Transform;
use crate::geometry::shape::mesh::{AsMesh, Mesh};
use crate::geometry::shape::nurbs::NurbsSurface;
use crate::geometry::shape::{cone::Cone, cube::Cube, cylinder::Cylinder, sphere::Sphere};
use crate::physics::collision::{apply_gravity, predict_position, resolve_collision};

use glutin::surface::WindowSurface;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyType {
    Static,  // 固定物体
    Dynamic, // 自由物体
}

#[derive(Clone, PartialEq)]
pub enum ShapeKind {
    Cube {
        width: f32,
        height: f32,
        depth: f32,
    },
    Sphere {
        radius: f32,
        sectors: u16,
    },
    Cylinder {
        top_radius: f32,
        bottom_radius: f32,
        height: f32,
        sectors: u16,
    },
    Cone {
        radius: f32,
        height: f32,
        sectors: u16,
    },
    Imported,
    Nurbs {
        degree: usize,
        control_points: Vec<[f32; 3]>,
        weights: Vec<f32>,
        u_count: usize,
        v_count: usize,
        current_nurbs_idx: usize,
    },
}

pub struct PhysicalProperties {
    pub velocity: [f32; 3],
    pub body_type: BodyType,
    pub restitution: f32,
}

pub struct RenderProperties {
    pub material: Material,
    pub visible: bool,
    pub use_texture: bool,
    pub selected_vertex_index: Option<usize>,
}

pub trait EditableMesh: AsMesh {
    fn ui(&mut self, ui: &imgui::Ui) -> bool {
        false
    }
}

impl<T> EditableMesh for T where T: AsMesh {}

pub struct GameObject {
    pub name: String,
    pub shape: Box<dyn EditableMesh>,
    pub mesh: Mesh, // cache of as_mesh
    pub rendering: RenderProperties,
    pub transform: Transform,
    pub physics: PhysicalProperties,
}

pub struct LightObject {
    pub name: String,
    pub light: Light,
}

pub struct CameraObject {
    pub name: String,
    pub camera: Camera,
}

impl GameObject {
    pub fn new(name: &str, shape: Box<dyn EditableMesh>, material: Material) -> Self {
        let mut obj = Self {
            name: name.to_string(),
            transform: Transform::default(),
            shape,
            mesh: Mesh {
                vertices: vec![],
                normals: vec![],
                tex_coords: vec![],
                indices: vec![],
                aabb: AABB::default(),
            },
            physics: PhysicalProperties {
                velocity: [0.0, 0.0, 0.0],
                body_type: BodyType::Static,
                restitution: 0.0,
            },
            rendering: RenderProperties {
                material,
                visible: true,
                use_texture: false,
                selected_vertex_index: None,
            },
        };
        obj.regenerate_mesh();
        obj
    }

    pub fn aabb(&self) -> AABB {
        let half_size = (self.mesh.aabb.max - self.mesh.aabb.min) / 2.0;
        AABB {
            min: self.transform.position - half_size,
            max: self.transform.position + half_size,
        }
    }

    pub fn set_body_type(&mut self, new_type: BodyType) {
        if self.physics.body_type == new_type {
            return;
        }
        self.physics.velocity = [0.0, 0.0, 0.0];
        self.physics.body_type = new_type;
    }
    pub fn regenerate_mesh(&mut self) {
        self.mesh = self.shape.as_mesh();
        // self.mesh = match &self.kind {
        //     ShapeKind::Cube {
        //         width,
        //         height,
        //         depth,
        //     } => {
        //         let s = Cube {
        //             width: *width,
        //             height: *height,
        //             depth: *depth,
        //         };
        //         s.as_mesh()
        //     }
        //     ShapeKind::Sphere { radius, sectors } => {
        //         let s = Sphere {
        //             radius: *radius,
        //             col_divisions: *sectors,
        //             row_divisions: *sectors,
        //         };
        //         s.as_mesh()
        //     }
        //     ShapeKind::Cylinder {
        //         top_radius,
        //         bottom_radius,
        //         height,
        //         sectors,
        //     } => {
        //         let s = Cylinder {
        //             bottom_radius: *bottom_radius,
        //             top_radius: *top_radius,
        //             height: *height,
        //             sectors: *sectors,
        //         };
        //         s.as_mesh()
        //     }
        //     ShapeKind::Cone {
        //         radius,
        //         height,
        //         sectors,
        //     } => {
        //         let s = Cone {
        //             radius: *radius,
        //             height: *height,
        //             sectors: *sectors,
        //         };
        //         s.as_mesh()
        //     }
        //     ShapeKind::Imported => self.mesh.clone(),
        //     ShapeKind::Nurbs {
        //         degree,
        //         control_points,
        //         weights,
        //         u_count,
        //         v_count,
        //         current_nurbs_idx: _,
        //     } => {
        //         let s = NurbsSurface {
        //             control_points: control_points.clone(),
        //             weights: weights.clone(),
        //             u_count: *u_count,
        //             v_count: *v_count,
        //             degree: *degree,
        //             splits: 32,
        //         };
        //         s.as_mesh()
        //     }
        // };
    }
}

pub struct World {
    pub last_frame_time: Instant,
    pub objects: Vec<GameObject>,
    pub selected_index: Option<usize>,
    pub lights: Vec<LightObject>,
    pub selected_light: Option<usize>,
    pub cameras: Vec<CameraObject>,
    pub selected_camera: Option<usize>,
    pub mouse_state: MouseState,
    pub default_aspect: f32,
    pub default_mat: Material,
    pub debug: bool,
    pub debug_frustum: bool,
    pub layer: usize,
    pub gravity: [f32; 3],
}

impl World {
    pub fn new() -> Self {
        Self {
            last_frame_time: Instant::now(),
            objects: Vec::new(),
            selected_index: None,
            lights: Vec::new(),
            selected_light: None,
            cameras: Vec::new(),
            selected_camera: None,
            mouse_state: MouseState::default(),
            default_aspect: 16.0 / 9.0,
            default_mat: Phong::new([1.0, 0.5, 0.31], [1.0, 0.5, 0.31], [0.5, 0.5, 0.5], 32.0)
                .to_material(),
            debug: false,
            debug_frustum: false,
            layer: 0,
            gravity: [0.0, -9.8, 0.0],
        }
    }

    pub fn step(&mut self, dt: f32) {
        for i in 0..self.objects.len() {
            if self.objects[i].physics.body_type != BodyType::Dynamic {
                continue;
            }
            apply_gravity(
                &mut self.objects[i],
                glam::f32::Vec3::from_array(self.gravity),
                dt,
            );

            let predicted_pos = predict_position(&self.objects[i], dt);

            let mut collided = false;
            for j in 0..self.objects.len() {
                if self.objects[j].physics.body_type == BodyType::Static {
                    let static_aabb = self.objects[j].aabb();
                    if resolve_collision(&mut self.objects[i], &static_aabb, predicted_pos) {
                        collided = true;
                        break;
                    }
                }
            }
            if !collided {
                self.objects[i].transform.position = predicted_pos;
            }
        }
    }

    pub fn handle_mouse_move(&mut self, delta: (f64, f64), window: &glium::winit::window::Window) {
        if let Some(idx) = self.get_selected_camera() {
            let mouse_state = &mut self.mouse_state;
            let camera = &mut self.cameras[idx].camera;
            if camera.move_state == crate::scene::camera::MoveState::Free
                && !mouse_state.is_locked()
            {
                mouse_state.toggle_lock(window);
            } else if mouse_state.is_locked()
                && camera.move_state != crate::scene::camera::MoveState::Free
            {
                mouse_state.toggle_lock(window);
            }
            mouse_state.handle_mouse_move(delta, camera, window);
        }
    }
    pub fn init_scene_1(&mut self, display: &glium::Display<WindowSurface>) {
        let (width, height) = display.get_framebuffer_dimensions();
        let aspect = width as f32 / height as f32;
        self.default_aspect = aspect;
        self.new_ambient_light("环境光", 0.1, [1.0, 1.0, 1.0]);
        self.new_camera("相机", aspect);
        let default_mat =
            Phong::new([1.0, 0.5, 0.31], [1.0, 0.5, 0.31], [0.5, 0.5, 0.5], 32.0).to_material();
        let mut floor = GameObject::new(
            "Floor",
            Box::new(Cube {
                width: 10.0,
                height: 0.1,
                depth: 10.0,
            }),
            default_mat,
        );
        floor.transform.position.y = -1.0;
        self.add_object(floor);

        let sphere = GameObject::new(
            "Sphere",
            Box::new(Sphere {
                radius: 0.8,
                col_divisions: 32,
                row_divisions: 32,
            }),
            default_mat,
        );
        self.add_object(sphere);
    }

    pub fn get_scene_bounding_box(&self) -> AABB {
        let mut aabb = AABB::default();
        for obj in &self.objects {
            let mesh_aabb = obj.mesh.aabb;
            let model_matrix = obj.transform.get_matrix();
            let mesh_aabb = mesh_aabb.get_global_aabb(model_matrix);
            aabb.union_aabb(&mesh_aabb);
        }
        return aabb;
    }
    pub fn new_camera(&mut self, name: &str, aspect: f32) {
        let camera = CameraObject {
            name: name.to_string(),
            camera: Camera::new(aspect),
        };
        self.add_camera(camera);
    }

    pub fn new_ambient_light(&mut self, name: &str, intensity: f32, color: [f32; 3]) {
        let light = LightObject {
            name: name.to_string(),
            light: AmbientLight { intensity, color }.to_light(),
        };
        self.add_light(light);
    }
    pub fn new_directional_light(
        &mut self,
        name: &str,
        position: [f32; 3],
        direction: [f32; 3],
        intensity: f32,
        color: [f32; 3],
    ) {
        let light = LightObject {
            name: name.to_string(),
            light: DirectionalLight {
                position,
                direction,
                intensity,
                color,
            }
            .to_light(),
        };
        self.add_light(light);
    }
    pub fn new_point_light(
        &mut self,
        name: &str,
        position: [f32; 3],
        intensity: f32,
        color: [f32; 3],
        kc: f32,
        kl: f32,
        kq: f32,
    ) {
        let light = LightObject {
            name: name.to_string(),
            light: PointLight {
                position,
                intensity,
                color,
                kc,
                kl,
                kq,
            }
            .to_light(),
        };
        self.add_light(light);
    }

    pub fn new_spot_light(
        &mut self,
        name: &str,
        position: [f32; 3],
        direction: [f32; 3],
        intensity: f32,
        color: [f32; 3],
        kc: f32,
        kl: f32,
        kq: f32,
        angle: f32,
    ) {
        let light = LightObject {
            name: name.to_string(),
            light: SpotLight {
                position,
                direction,
                intensity,
                color,
                kc,
                kl,
                kq,
                angle,
            }
            .to_light(),
        };
        self.add_light(light);
    }

    pub fn add_camera(&mut self, camera: CameraObject) {
        self.cameras.push(camera);
        self.selected_camera = Some(self.cameras.len() - 1);
    }

    pub fn get_selected_camera(&self) -> Option<usize> {
        if let Some(idx) = self.selected_camera {
            if idx < self.cameras.len() {
                return Some(idx);
            }
        }
        None
    }

    pub fn add_light(&mut self, light: LightObject) {
        self.lights.push(light);
        self.selected_light = Some(self.lights.len() - 1);
    }

    pub fn get_selected_light(&mut self) -> Option<&mut LightObject> {
        if let Some(idx) = self.selected_light {
            if idx < self.lights.len() {
                return Some(&mut self.lights[idx]);
            }
        }
        None
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
