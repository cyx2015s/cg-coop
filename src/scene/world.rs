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
    Static,
    Dynamic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionBehavior {
    None,
    Door { is_open: bool, base_yaw: f32 },
    Window { is_broken: bool },
}

#[derive(Clone, PartialEq)]
pub enum ShapeKind {
    Cube, Sphere, Cylinder, Cone, Imported, Nurbs,
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
            restitution: 0.5,
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
    fn ui(&mut self, ui: &imgui::Ui) -> bool { false }
    fn debug_ui(&mut self, _ui: &imgui::Ui) {}
}

pub struct GameObject {
    pub name: String,
    pub shape: Box<dyn EditableMesh>,
    pub mesh: Mesh,
    pub rendering: RenderProperties,
    pub transform: Transform,
    pub physics: PhysicalProperties,
    pub behavior: InteractionBehavior,
}

pub struct LightObject {
    pub name: String,
    pub light: Light,
}

pub struct CameraObject {
    pub name: String,
    pub camera: Camera,
}

// RigidBody 实现
impl RigidBody for CameraObject {
    fn transform(&self) -> &Transform { &self.camera.transform }
    fn transform_mut(&mut self) -> &mut Transform { &mut self.camera.transform }
    fn velocity(&self) -> [f32; 3] { self.camera.physics.velocity }
    fn velocity_mut(&mut self) -> &mut [f32; 3] { &mut self.camera.physics.velocity }
    fn mass(&self) -> f32 { self.camera.physics.mass }
    fn force(&self) -> [f32; 3] { self.camera.physics.force }
    fn friction(&self) -> [f32; 3] { self.camera.physics.friction }
    fn body_type(&self) -> BodyType { self.camera.physics.body_type }
    fn restitution(&self) -> f32 { self.camera.physics.restitution }
    fn aabb(&self) -> AABB {
        AABB {
            min: glam::Vec3::new(-0.3, -1.5, -0.3),
            max: glam::Vec3::new(0.3, 1.5, 0.3),
        }.get_global_aabb(self.camera.transform.get_matrix())
    }
}

impl RigidBody for GameObject {
    fn transform(&self) -> &Transform { &self.transform }
    fn transform_mut(&mut self) -> &mut Transform { &mut self.transform }
    fn velocity(&self) -> [f32; 3] { self.physics.velocity }
    fn velocity_mut(&mut self) -> &mut [f32; 3] { &mut self.physics.velocity }
    fn body_type(&self) -> BodyType { self.physics.body_type }
    fn restitution(&self) -> f32 { self.physics.restitution }
    fn force(&self) -> [f32; 3] { self.physics.force }
    fn friction(&self) -> [f32; 3] { self.physics.friction }
    fn mass(&self) -> f32 { self.physics.mass }
    fn aabb(&self) -> AABB {
        self.mesh.aabb.get_global_aabb(self.transform.get_matrix())
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
                // bvh: None, 
            },
            physics: PhysicalProperties::default(),
            rendering: RenderProperties {
                material,
                visible: true,
                use_texture: false,
                selected_vertex_index: None,
            },
            behavior: InteractionBehavior::None,
        };
        obj.regenerate_mesh();
        obj
    }

    pub fn set_body_type(&mut self, new_type: BodyType) {
        if self.physics.body_type == new_type { return; }
        self.physics.velocity = [0.0, 0.0, 0.0];
        self.physics.body_type = new_type;
    }

    pub fn regenerate_mesh(&mut self) {
        self.mesh = self.shape.as_mesh();
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
    fn default() -> Self { Self::new() }
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
        for obj in &mut self.objects {
            if let InteractionBehavior::Door { is_open, base_yaw } = obj.behavior {
                let target_yaw = if is_open { base_yaw + 1.5708 } else { base_yaw };
                let current_rot = obj.transform.rotation;
                let target_rot = glam::f32::Quat::from_rotation_y(target_yaw);
                obj.transform.rotation = current_rot.slerp(target_rot, dt * 5.0);
            }
        }

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


    pub fn handle_interaction_input(&mut self, player_pos: glam::f32::Vec3) {
        let mut nearest_idx = None;
        let mut min_dist = 3.0;

        for (i, obj) in self.objects.iter().enumerate() {
            if !obj.rendering.visible { continue; }
            let dist = obj.transform.position.distance(player_pos);
            if dist < min_dist {
                min_dist = dist;
                nearest_idx = Some(i);
            }
        }

        if let Some(idx) = nearest_idx {
            let behavior = self.objects[idx].behavior;
            match behavior {
                InteractionBehavior::Door { is_open, base_yaw } => {
                    self.objects[idx].behavior = InteractionBehavior::Door { is_open: !is_open, base_yaw };
                }
                InteractionBehavior::Window { is_broken } => {
                    if !is_broken { self.break_window(idx); }
                }
                _ => {}
            }
        }
    }

    fn break_window(&mut self, window_idx: usize) {
        let window = &mut self.objects[window_idx];
        window.rendering.visible = false;
        if let InteractionBehavior::Window { is_broken } = &mut window.behavior {
            *is_broken = true;
        }

        let pos = window.transform.position;
        let size = 1.0; 
        let step = size / 4.0;
        let mut shards = Vec::new();
        
        for r in 0..4 {
            for c in 0..4 {
                let offset = glam::vec3(
                    (c as f32 * step) - size/2.0 + step/2.0,
                    (r as f32 * step) - size/2.0 + step/2.0,
                    0.0
                );
                let mut shard = GameObject::new(
                    "Shard",
                    Box::new(Cube { width: step*0.9, height: step*0.9, depth: 0.05 }), 
                    window.rendering.material
                );
                shard.transform.position = pos + offset;
                shard.physics.body_type = BodyType::Dynamic;
                shard.physics.velocity = [(c as f32 - 1.5) * 2.0, (r as f32 - 1.5) * 2.0, 5.0];
                shards.push(shard);
            }
        }
        self.objects.append(&mut shards);
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
    pub fn create_door(&mut self, pos: glam::f32::Vec3) {
        let width = 1.0;
        let height = 2.0;
        let mut door = GameObject::new(
            "Door",
            Box::new(Cube { width, height, depth: 0.1 }),
            self.default_mat
        );
        let offset = glam::vec3(width / 2.0, 0.0, 0.0);
        for v in &mut door.mesh.vertices {
            v[0] += offset.x; v[1] += offset.y; v[2] += offset.z;
        }
        door.transform.position = pos;
        door.behavior = InteractionBehavior::Door { is_open: false, base_yaw: 0.0 };
        door.set_body_type(BodyType::Static);
        self.add_object(door);
    }

    pub fn create_window(&mut self, pos: glam::f32::Vec3) {
        let mut win = GameObject::new(
            "Window",
            Box::new(Cube { width: 2.0, height: 1.5, depth: 0.05 }),
            self.default_mat
        );
        win.transform.position = pos;
        win.behavior = InteractionBehavior::Window { is_broken: false };
        win.set_body_type(BodyType::Static);
        self.add_object(win);
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
        if let Some(idx) = self.selected_camera && idx < self.cameras.len() { Some(idx) } else { None }
    }
    pub fn add_light(&mut self, light: LightObject) {
        self.lights.push(light);
        self.selected_light = Some(self.lights.len() - 1);
    }
    pub fn get_selected_light(&mut self) -> Option<&mut LightObject> {
        if let Some(idx) = self.selected_light && idx < self.lights.len() { Some(&mut self.lights[idx]) } else { None }
    }
    pub fn add_object(&mut self, obj: GameObject) {
        self.objects.push(obj);
        self.selected_index = Some(self.objects.len() - 1);
    }
    pub fn get_selected_mut(&mut self) -> Option<&mut GameObject> {
        if let Some(idx) = self.selected_index && idx < self.objects.len() { Some(&mut self.objects[idx]) } else { None }
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
            Box::new(Cube { width: 10.0, height: 0.1, depth: 10.0 }),
            default_mat,
        );
        floor.transform.position.y = -1.0;
        self.add_object(floor);

        let sphere = GameObject::new(
            "Sphere",
            Box::new(Sphere { radius: 0.8, col_divisions: 32, row_divisions: 32 }),
            default_mat,
        );
        self.add_object(sphere);
    }
    
    pub fn get_scene_bounding_box(&self) -> AABB {
        let mut aabb = AABB::default();
        for obj in &self.objects {
            aabb.union_aabb(&obj.aabb());
        }
        aabb
    }

    pub fn handle_mouse_move(&mut self, delta: (f64, f64), window: &glium::winit::window::Window) {
        if let Some(idx) = self.get_selected_camera() {
            let mouse_state = &mut self.mouse_state;
            let camera = &mut self.cameras[idx].camera;
            
            // 处理鼠标锁定状态逻辑
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
            
            // 转发给 mouse_state 处理具体的旋转
            mouse_state.handle_mouse_move(delta, camera, window);
        }
    }
}