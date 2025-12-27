use super::camera::{Camera, MouseState};
use super::light::Light;

use crate::core::material::Material;
use crate::core::math::aabb::AABB;
use crate::core::math::transform::Transform;
use crate::geometry::shape::mesh::{AsMesh, Mesh};
use crate::geometry::shape::nurbs::NurbsSurface;
use crate::geometry::shape::{cone::Cone, cube::Cube, cylinder::Cylinder, sphere::Sphere};
use crate::physics::collision::{apply_gravity, predict_position, resolve_collision};
use crate::physics::rigid::RigidBody;

use glutin::surface::WindowSurface;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyHandle {
    Object(usize),
    Camera(usize),
}

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
    pub force: [f32; 3],
    pub friction: [f32; 3],
    pub mass: f32,
    pub body_type: BodyType,
    pub restitution: f32,
}

impl Default for PhysicalProperties {
    fn default() -> Self {
        Self {
            velocity: [0.0, 0.0, 0.0],
            force: [0.0, 0.0, 0.0],
            friction: [0.0, 0.0, 0.0],
            mass: 1.0,
            body_type: BodyType::Static,
            restitution: 0.0,
        }
    }
}

pub struct RenderProperties {
    pub material: Material,
    pub visible: bool,
    pub use_texture: bool,
    pub selected_vertex_index: Option<usize>,
}

pub trait EditableMesh: AsMesh {
    /// 返回值：是否需要重新生成网格
    fn ui(&mut self, ui: &imgui::Ui) -> bool {
        false
    }
    /// 物体渲染分为两步，基础的和调试的
    fn debug_ui(&mut self, _ui: &imgui::Ui) {}
}

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

impl RigidBody for CameraObject {
    fn transform(&self) -> &Transform {
        &self.camera.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.camera.transform
    }

    fn velocity(&self) -> [f32; 3] {
        self.camera.physics.velocity
    }

    fn velocity_mut(&mut self) -> &mut [f32; 3] {
        &mut self.camera.physics.velocity
    }

    fn mass(&self) -> f32 {
        self.camera.physics.mass
    }

    fn force(&self) -> [f32; 3] {
        self.camera.physics.force
    }

    fn friction(&self) -> [f32; 3] {
        self.camera.physics.friction
    }

    fn body_type(&self) -> BodyType {
        self.camera.physics.body_type
    }

    fn restitution(&self) -> f32 {
        self.camera.physics.restitution
    }

    fn aabb(&self) -> AABB {
        AABB {
            min: glam::Vec3::new(-0.3, -0.5, -0.3),
            max: glam::Vec3::new(0.3, 0.5, 0.3),
        }
        .get_global_aabb(self.camera.transform.get_matrix())
    }
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
            physics: PhysicalProperties::default(),
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

impl RigidBody for GameObject {
    fn transform(&self) -> &Transform {
        &self.transform
    }
    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    fn velocity(&self) -> [f32; 3] {
        self.physics.velocity
    }
    fn velocity_mut(&mut self) -> &mut [f32; 3] {
        &mut self.physics.velocity
    }

    fn body_type(&self) -> BodyType {
        self.physics.body_type
    }

    fn restitution(&self) -> f32 {
        self.physics.restitution
    }

    fn force(&self) -> [f32; 3] {
        self.physics.force
    }

    fn friction(&self) -> [f32; 3] {
        self.physics.friction
    }
    fn aabb(&self) -> AABB {
        self.mesh.aabb.get_global_aabb(self.transform.get_matrix())
    }

    fn mass(&self) -> f32 {
        self.physics.mass
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
    pub camera_force: [bool; 6],
    pub layer: usize,
    pub gravity: [f32; 3],
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
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
            default_mat: Material::PHONG,
            debug: false,
            debug_frustum: false,
            layer: 0,
            camera_force: [false; 6],
            gravity: [0.0, -9.8, 0.0],
        }
    }

    pub fn step(&mut self, dt: f32) {
        let mut bodies: Vec<BodyHandle> = Vec::new();
        for i in 0..self.objects.len() {
            bodies.push(BodyHandle::Object(i));
        }
        if let Some(idx) = self.get_selected_camera() {
            bodies.push(BodyHandle::Camera(idx));
        }
        let gravity = glam::f32::Vec3::from_array(self.gravity);
        for i in 0..bodies.len() {
            let mut collided = false;
            for j in 0..bodies.len() {
                if i == j {
                    continue;
                }
                let (a, b) = self.get_two_bodies_mut(bodies[i], bodies[j]);
                if !a.is_dynamic() {
                    break;
                }
                apply_gravity(a, gravity, dt);

                if resolve_collision(a, b, dt) {
                    collided = true;
                    break;
                }
            }

            if !collided {
                match bodies[i] {
                    BodyHandle::Object(i) => {
                        let predicted_pos = predict_position(&self.objects[i], dt);
                        self.objects[i].transform.position = predicted_pos;
                    }
                    BodyHandle::Camera(i) => {
                        let predicted_pos = predict_position(&self.cameras[i], dt);
                        self.cameras[i].camera.transform.position = predicted_pos;
                    }
                }
            }

            match bodies[i] {
                BodyHandle::Camera(i) => {
                    self.cameras[i].update_velocity(dt);
                    let predicted_pos = predict_position(&self.cameras[i], dt);
                    self.cameras[i].camera.transform.position = predicted_pos;
                }
                BodyHandle::Object(_i) => {}
            }
        }
    }

    fn get_two_bodies_mut(
        &mut self,
        a: BodyHandle,
        b: BodyHandle,
    ) -> (&mut dyn RigidBody, &mut dyn RigidBody) {
        match (a, b) {
            (BodyHandle::Object(i), BodyHandle::Object(j)) => {
                if i < j {
                    let (left, right) = self.objects.split_at_mut(i + 1);
                    (&mut left[i], &mut right[j - i - 1])
                } else {
                    let (left, right) = self.objects.split_at_mut(j + 1);
                    (&mut right[i - j - 1], &mut left[j])
                }
            }

            (BodyHandle::Camera(i), BodyHandle::Camera(j)) => {
                let (left, right) = self.cameras.split_at_mut(j);
                (&mut left[i], &mut right[0])
            }

            (BodyHandle::Object(i), BodyHandle::Camera(j)) => {
                let obj = &mut self.objects[i];
                let cam = &mut self.cameras[j];
                (obj, cam)
            }

            (BodyHandle::Camera(i), BodyHandle::Object(j)) => {
                let cam = &mut self.cameras[i];
                let obj = &mut self.objects[j];
                (cam, obj)
            }
        }
    }
    pub fn handle_mouse_move(&mut self, delta: (f64, f64), window: &glium::winit::window::Window) {
        if let Some(idx) = self.get_selected_camera() {
            let mouse_state = &mut self.mouse_state;
            let camera = &mut self.cameras[idx].camera;
            if (camera.move_state == crate::scene::camera::MoveState::Free
                || camera.move_state == crate::scene::camera::MoveState::RigidBody)
                && !mouse_state.is_locked()
            {
                mouse_state.toggle_lock(window);
            } 
            else if mouse_state.is_locked()
                && (camera.move_state != crate::scene::camera::MoveState::Free
                    && camera.move_state != crate::scene::camera::MoveState::RigidBody)
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
        self.new_ambient_light("环境光");
        self.new_camera("相机", aspect);
        let default_mat = Material::PHONG;
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
        aabb
    }
    pub fn new_camera(&mut self, name: &str, aspect: f32) {
        let mut camera = CameraObject {
            name: name.to_string(),
            camera: Camera::new(aspect),
        };
        camera.camera.init();
        self.add_camera(camera);
    }

    pub fn new_ambient_light(&mut self, name: &str) {
        let light = LightObject {
            name: name.to_string(),
            light: Light::AMBIENT,
        };
        self.add_light(light);
    }
    pub fn new_directional_light(&mut self, name: &str) {
        let light = LightObject {
            name: name.to_string(),
            light: Light::DERECTIONAL,
        };
        self.add_light(light);
    }

    pub fn new_point_light(&mut self, name: &str) {
        let light = LightObject {
            name: name.to_string(),
            light: Light::POINT,
        };
        self.add_light(light);
    }

    pub fn new_spot_light(&mut self, name: &str) {
        let light = LightObject {
            name: name.to_string(),
            light: Light::SPOT,
        };
        self.add_light(light);
    }

    pub fn add_camera(&mut self, camera: CameraObject) {
        self.cameras.push(camera);
        self.selected_camera = Some(self.cameras.len() - 1);
    }

    pub fn get_selected_camera(&self) -> Option<usize> {
        if let Some(idx) = self.selected_camera
            && idx < self.cameras.len()
        {
            return Some(idx);
        }
        None
    }

    pub fn add_light(&mut self, light: LightObject) {
        self.lights.push(light);
        self.selected_light = Some(self.lights.len() - 1);
    }

    pub fn get_selected_light(&mut self) -> Option<&mut LightObject> {
        if let Some(idx) = self.selected_light
            && idx < self.lights.len()
        {
            return Some(&mut self.lights[idx]);
        }
        None
    }
    pub fn add_object(&mut self, obj: GameObject) {
        self.objects.push(obj);
        self.selected_index = Some(self.objects.len() - 1);
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut GameObject> {
        if let Some(idx) = self.selected_index
            && idx < self.objects.len()
        {
            return Some(&mut self.objects[idx]);
        }
        None
    }
}
