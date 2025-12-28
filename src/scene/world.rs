use super::camera::{Camera, MouseState};
use super::light::Light;

use crate::core::material::Material;
use crate::core::math::aabb::AABB;
use crate::core::math::transform::Transform;
use crate::geometry::shape::mesh::{AsMesh, Mesh};
use crate::geometry::shape::nurbs::NurbsSurface;
use crate::geometry::shape::{cone::Cone, cube::Cube, cylinder::Cylinder, sphere::Sphere};
use crate::physics::collision::{apply_gravity, resolve_collision};
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
            min: glam::Vec3::new(-0.25, -0.85, -0.25),
            max: glam::Vec3::new(0.25, 0.85, 0.25),
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
        // 1. 处理门窗动画 
        for obj in &mut self.objects {
            if let InteractionBehavior::Door { is_open, base_yaw } = obj.behavior {
                let target_yaw = if is_open { base_yaw + 1.5708 } else { base_yaw };
                let current_rot = obj.transform.rotation;
                let target_rot = glam::f32::Quat::from_rotation_y(target_yaw);
                obj.transform.rotation = current_rot.slerp(target_rot, dt * 5.0);
            }
        }

        // 2. 收集所有物体
        let mut bodies: Vec<BodyHandle> = Vec::new();
        for i in 0..self.objects.len() {
            bodies.push(BodyHandle::Object(i));
        }
        if let Some(idx) = self.get_selected_camera() {
            bodies.push(BodyHandle::Camera(idx));
        }

        let gravity = glam::f32::Vec3::from_array(self.gravity);

        for i in 0..bodies.len() {
            let is_dynamic = match bodies[i] {
                BodyHandle::Object(idx) => self.objects[idx].physics.body_type == BodyType::Dynamic,
                BodyHandle::Camera(idx) => self.cameras[idx].camera.physics.body_type == BodyType::Dynamic,
            };

            if !is_dynamic { continue; }

            match bodies[i] {
                BodyHandle::Object(idx) => {
                    let obj = &mut self.objects[idx];
                    apply_gravity(obj, gravity, dt);
                    obj.update_velocity(dt);
                    let vel = glam::f32::Vec3::from(obj.physics.velocity);
                    obj.transform.position += vel * dt;
                }
                BodyHandle::Camera(idx) => {
                    let cam = &mut self.cameras[idx];
                    if cam.camera.move_state == crate::scene::camera::MoveState::RigidBody {
                        apply_gravity(cam, gravity, dt);
                    }
                    cam.update_velocity(dt);
                    let vel = glam::f32::Vec3::from(cam.camera.physics.velocity);
                    cam.camera.transform.position += vel * dt;
                }
            }

            for j in 0..bodies.len() {
                if i == j { continue; }
                
                let (a, b) = self.get_two_bodies_mut(bodies[i], bodies[j]);

                if !b.is_dynamic() {
                    resolve_collision(a, b, dt);

                }
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

    pub fn init_house_scene(&mut self, _display: &glium::Display<glutin::surface::WindowSurface>) {
        // 清理
        self.objects.clear();
        self.lights.clear();
        self.cameras.clear();

        // 材质库
        let floor_mat = Material { ka: [0.15, 0.15, 0.18], kd: [0.25, 0.25, 0.3], ks: [0.1, 0.1, 0.1], ns: 32.0, ..Material::default() };
        let wall_mat = Material { ka: [0.5, 0.5, 0.52], kd: [0.65, 0.65, 0.67], ks: [0.05, 0.05, 0.05], ns: 10.0, ..Material::default() };
        let trim_mat = Material { ka: [0.15, 0.1, 0.05], kd: [0.3, 0.2, 0.1], ks: [0.1, 0.1, 0.1], ns: 10.0, ..Material::default() }; // 深棕色边框
        let door_mat = Material { ka: [0.45, 0.25, 0.1], kd: [0.6, 0.35, 0.15], ks: [0.2, 0.2, 0.2], ns: 24.0, ..Material::default() };
        let glass_mat = Material { ka: [0.1, 0.2, 0.3], kd: [0.2, 0.4, 0.6], ks: [0.9, 0.9, 0.9], ns: 128.0, ..Material::default() }; // 简单的玻璃蓝
        let screen_mat = Material { ka: [0.05, 0.05, 0.1], kd: [0.1, 0.1, 0.2], ks: [0.5, 0.5, 0.6], ns: 64.0, ..Material::default() };

        // 关键尺寸定义 
        let room_w = 20.0;
        let room_h = 6.0;
        let room_d = 25.0;
        let wall_thick = 0.4; 

        // 门参数
        let door_w = 2.4;
        let door_h = 2.8;
        let door_frame_thick = 0.15;
        let door_total_w = door_w + door_frame_thick * 2.0; 
        let door_total_h = door_h + door_frame_thick;      

        // 窗参数
        let win_w = 4.0;
        let win_h = 2.5;
        let win_y = 2.5; 
        let win_frame_thick = 0.15;
        let win_total_w = win_w + win_frame_thick * 2.0;
        let win_total_h = win_h + win_frame_thick * 2.0;
        
        // 地板
        let mut floor = GameObject::new("Floor_Main", Box::new(Cube { width: 60.0, height: 0.5, depth: 60.0 }), floor_mat);
        floor.transform.position = [0.0, -0.25, 0.0].into(); 
        floor.set_body_type(BodyType::Static);
        self.add_object(floor);

        // B. 天花板 
        let mut ceiling = GameObject::new("Ceiling", Box::new(Cube { width: room_w + 2.0, height: 0.5, depth: room_d + 2.0 }), wall_mat);
        ceiling.transform.position = [0.0, room_h + 0.25, 0.0].into();
        ceiling.set_body_type(BodyType::Static);
        self.add_object(ceiling);

        // 后墙 (实心) 
        let mut wall_back = GameObject::new("Wall_Back", Box::new(Cube { width: room_w + wall_thick*2.0, height: room_h, depth: wall_thick }), wall_mat);
        wall_back.transform.position = [0.0, room_h/2.0, room_d/2.0 + wall_thick/2.0].into(); // 往外推半个墙厚
        wall_back.set_body_type(BodyType::Static);
        self.add_object(wall_back);

        // 右墙 (实心)
        let mut wall_right = GameObject::new("Wall_Right", Box::new(Cube { width: wall_thick, height: room_h, depth: room_d }), wall_mat);
        wall_right.transform.position = [room_w/2.0 + wall_thick/2.0, room_h/2.0, 0.0].into();
        wall_right.set_body_type(BodyType::Static);
        self.add_object(wall_right);

        // 前墙 (带门洞)
        
        // 门头墙
        let header_h = room_h - door_total_h;
        let mut wall_front_header = GameObject::new("Wall_Front_Header", Box::new(Cube { width: door_total_w, height: header_h, depth: wall_thick }), wall_mat);
        wall_front_header.transform.position = [0.0, door_total_h + header_h/2.0, -room_d/2.0 - wall_thick/2.0].into();
        wall_front_header.set_body_type(BodyType::Static);
        self.add_object(wall_front_header);

        // 左右两块大墙
        let front_side_w = (room_w - door_total_w) / 2.0 + wall_thick;
        let front_pos_x = door_total_w/2.0 + front_side_w/2.0 - wall_thick; 

        let mut wall_front_l = GameObject::new("Wall_Front_L", Box::new(Cube { width: front_side_w, height: room_h, depth: wall_thick }), wall_mat);
        wall_front_l.transform.position = [-front_pos_x - 0.2, room_h/2.0, -room_d/2.0 - wall_thick/2.0].into(); // -0.2 是修正重叠量
        wall_front_l.set_body_type(BodyType::Static);
        self.add_object(wall_front_l);

        let mut wall_front_r = GameObject::new("Wall_Front_R", Box::new(Cube { width: front_side_w, height: room_h, depth: wall_thick }), wall_mat);
        wall_front_r.transform.position = [front_pos_x + 0.2, room_h/2.0, -room_d/2.0 - wall_thick/2.0].into();
        wall_front_r.set_body_type(BodyType::Static);
        self.add_object(wall_front_r);


        // 左墙 (带窗洞) 
        
        // 窗下墙
        let win_bottom_h = win_y - win_total_h/2.0;
        let mut wall_left_bot = GameObject::new("Wall_Left_Bot", Box::new(Cube { width: wall_thick, height: win_bottom_h, depth: win_total_w }), wall_mat);
        wall_left_bot.transform.position = [-room_w/2.0 - wall_thick/2.0, win_bottom_h/2.0, 0.0].into();
        wall_left_bot.set_body_type(BodyType::Static);
        self.add_object(wall_left_bot);

        // 窗上墙
        let win_top_start = win_y + win_total_h/2.0;
        let win_top_h = room_h - win_top_start;
        let mut wall_left_top = GameObject::new("Wall_Left_Top", Box::new(Cube { width: wall_thick, height: win_top_h, depth: win_total_w }), wall_mat);
        wall_left_top.transform.position = [-room_w/2.0 - wall_thick/2.0, win_top_start + win_top_h/2.0, 0.0].into();
        wall_left_top.set_body_type(BodyType::Static);
        self.add_object(wall_left_top);

        // 窗两侧墙
        let side_wall_d = (room_d - win_total_w) / 2.0;
        let side_pos_z = win_total_w/2.0 + side_wall_d/2.0;
        
        // 左墙-前段
        let mut wall_left_front = GameObject::new("Wall_Left_Front", Box::new(Cube { width: wall_thick, height: room_h, depth: side_wall_d }), wall_mat);
        wall_left_front.transform.position = [-room_w/2.0 - wall_thick/2.0, room_h/2.0, -side_pos_z].into();
        wall_left_front.set_body_type(BodyType::Static);
        self.add_object(wall_left_front);

        // 左墙-后段
        let mut wall_left_back = GameObject::new("Wall_Left_Back", Box::new(Cube { width: wall_thick, height: room_h, depth: side_wall_d }), wall_mat);
        wall_left_back.transform.position = [-room_w/2.0 - wall_thick/2.0, room_h/2.0, side_pos_z].into();
        wall_left_back.set_body_type(BodyType::Static);
        self.add_object(wall_left_back);

        // 门框与门
        let door_base_z = -room_d/2.0 - wall_thick/2.0;
        // 门楣框
        let mut d_frame_top = GameObject::new("DoorFrame_Top", Box::new(Cube{width: door_total_w, height: door_frame_thick, depth: wall_thick + 0.1}), trim_mat);
        d_frame_top.transform.position = [0.0, door_h + door_frame_thick/2.0, door_base_z].into();
        self.add_object(d_frame_top);
        // 门左框
        let mut d_frame_l = GameObject::new("DoorFrame_L", Box::new(Cube{width: door_frame_thick, height: door_total_h, depth: wall_thick + 0.1}), trim_mat);
        d_frame_l.transform.position = [-door_w/2.0 - door_frame_thick/2.0, door_total_h/2.0, door_base_z].into();
        self.add_object(d_frame_l);
        // 门右框
        let mut d_frame_r = GameObject::new("DoorFrame_R", Box::new(Cube{width: door_frame_thick, height: door_total_h, depth: wall_thick + 0.1}), trim_mat);
        d_frame_r.transform.position = [door_w/2.0 + door_frame_thick/2.0, door_total_h/2.0, door_base_z].into();
        self.add_object(d_frame_r);
        
        // 门板 
        let mut door = GameObject::new("Door", Box::new(Cube{width: door_w, height: door_h, depth: 0.15}), door_mat);
        let offset = glam::vec3(door_w/2.0, 0.0, 0.0);
        for v in &mut door.mesh.vertices { v[0] += offset.x; v[1] += offset.y; v[2] += offset.z; }
        door.transform.position = [-door_w/2.0, door_h/2.0, door_base_z].into();
        door.behavior = InteractionBehavior::Door { is_open: false, base_yaw: 0.0 };
        door.set_body_type(BodyType::Static);
        self.add_object(door);

        // 窗框与窗
        let win_base_x = -room_w/2.0 - wall_thick/2.0;
        // 上下框
        let mut w_frame_top = GameObject::new("WinFrame_Top", Box::new(Cube{width: wall_thick + 0.1, height: win_frame_thick, depth: win_total_w}), trim_mat);
        w_frame_top.transform.position = [win_base_x, win_y + win_h/2.0 + win_frame_thick/2.0, 0.0].into();
        self.add_object(w_frame_top);

        let mut w_frame_bot = GameObject::new("WinFrame_Bot", Box::new(Cube{width: wall_thick + 0.1, height: win_frame_thick, depth: win_total_w}), trim_mat);
        w_frame_bot.transform.position = [win_base_x, win_y - win_h/2.0 - win_frame_thick/2.0, 0.0].into();
        self.add_object(w_frame_bot);

        // 左右框
        let mut w_frame_l = GameObject::new("WinFrame_L", Box::new(Cube{width: wall_thick + 0.1, height: win_total_h, depth: win_frame_thick}), trim_mat);
        w_frame_l.transform.position = [win_base_x, win_y, -win_w/2.0 - win_frame_thick/2.0].into();
        self.add_object(w_frame_l);

        let mut w_frame_r = GameObject::new("WinFrame_R", Box::new(Cube{width: wall_thick + 0.1, height: win_total_h, depth: win_frame_thick}), trim_mat);
        w_frame_r.transform.position = [win_base_x, win_y, win_w/2.0 + win_frame_thick/2.0].into();
        self.add_object(w_frame_r);

        // 玻璃
        let mut window = GameObject::new("Window_Glass", Box::new(Cube{width: 0.05, height: win_h, depth: win_w}), glass_mat);
        window.transform.position = [win_base_x, win_y, 0.0].into();
        window.behavior = InteractionBehavior::Window { is_broken: false };
        window.set_body_type(BodyType::Static);
        self.add_object(window);
        
        // NURBS 屏幕
        let mut control_points = Vec::new();
        let screen_w = 12.0; let screen_h = 8.0;
        for row in 0..4 {
            let y = -screen_h/2.0 + row as f32 * (screen_h/3.0) + 2.0;
            for col in 0..5 {
                let x = -screen_w/2.0 + col as f32 * (screen_w/4.0);
                let z = room_d/2.0 - 1.5 + (col as f32 - 2.0).powi(2) * 0.2; // 抛物线弯曲
                control_points.push([x, y, z]);
            }
        }
        let nurbs = NurbsSurface { control_points, weights: vec![1.0; 20], u_count: 5, v_count: 4, degree: 2, splits: 20, selected_point_idx: 0, u_knots: vec![], v_knots: vec![] };
        let mut screen = GameObject::new("Screen", Box::new(nurbs), screen_mat);
        screen.set_body_type(BodyType::Static);
        self.add_object(screen);
        
        // 定义装饰材质
        let metal = Material { 
            ka: [0.25, 0.25, 0.3], 
            kd: [0.45, 0.45, 0.55], 
            ks: [0.85, 0.85, 0.95], 
            ns: 80.0, 
            ..Material::default() 
        };
        let lamp_mat = Material { 
            ka: [0.85, 0.85, 0.8], 
            kd: [0.95, 0.95, 0.88], 
            ks: [0.4, 0.4, 0.4], 
            ns: 32.0, 
            ..Material::default() 
        };
        let bulb_mat = Material { 
            ka: [1.0, 0.95, 0.7], 
            kd: [1.0, 1.0, 0.9], 
            ks: [0.9, 0.9, 0.9], 
            ns: 128.0, 
            ..Material::default() 
        };

        // 天花板横梁 
        for i in 0..5 {
            let mut beam = GameObject::new(
                &format!("Ceiling_Beam_{}", i),
                Box::new(Cylinder { bottom_radius: 0.18, top_radius: 0.18, height: room_w, sectors: 16 }),
                trim_mat
            );
            beam.transform.position = [0.0, room_h - 0.2, -room_d/2.0 + 3.0 + i as f32 * 4.5].into();
            beam.transform.rotation = glam::Quat::from_rotation_z(std::f32::consts::PI / 2.0);
            self.add_object(beam);
        }

        // 吊灯系统
        let lamp1_pos = glam::vec3(0.0, 4.7, -5.0);
        
        let mut lampshade1 = GameObject::new(
            "Lamp1_Shade",
            Box::new(Cone { radius: 0.4, height: 0.6, sectors: 20 }),
            lamp_mat
        );
        lampshade1.transform.position = lamp1_pos;
        lampshade1.transform.rotation = glam::Quat::from_rotation_x(std::f32::consts::PI);
        self.add_object(lampshade1);

        let mut rod1 = GameObject::new(
            "Lamp1_Rod",
            Box::new(Cylinder { bottom_radius: 0.025, top_radius: 0.025, height: 1.0, sectors: 12 }),
            metal
        );
        rod1.transform.position = lamp1_pos + glam::vec3(0.0, 0.8, 0.0);
        self.add_object(rod1);

        let mut bulb1 = GameObject::new(
            "Lamp1_Bulb",
            Box::new(Sphere { radius: 0.15, col_divisions: 20, row_divisions: 20 }),
            bulb_mat
        );
        bulb1.transform.position = lamp1_pos + glam::vec3(0.0, -0.25, 0.0);
        self.add_object(bulb1);

        // 后方吊灯（靶场照明）
        let lamp2_pos = glam::vec3(0.0, 4.2, 8.0);
        
        let mut lampshade2 = GameObject::new(
            "Lamp2_Shade",
            Box::new(Cone { radius: 0.5, height: 0.7, sectors: 20 }),
            lamp_mat
        );
        lampshade2.transform.position = lamp2_pos;
        lampshade2.transform.rotation = glam::Quat::from_rotation_x(std::f32::consts::PI);
        self.add_object(lampshade2);

        let mut rod2 = GameObject::new(
            "Lamp2_Rod",
            Box::new(Cylinder { bottom_radius: 0.03, top_radius: 0.03, height: 1.5, sectors: 12 }),
            metal
        );
        rod2.transform.position = lamp2_pos + glam::vec3(0.0, 1.1, 0.0);
        self.add_object(rod2);

        let mut bulb2 = GameObject::new(
            "Lamp2_Bulb",
            Box::new(Sphere { radius: 0.18, col_divisions: 20, row_divisions: 20 }),
            bulb_mat
        );
        bulb2.transform.position = lamp2_pos + glam::vec3(0.0, -0.3, 0.0);
        self.add_object(bulb2);

        // 装饰柱子 
        let mut pillar1 = GameObject::new(
            "Pillar_FrontRight",
            Box::new(Cylinder { bottom_radius: 0.3, top_radius: 0.28, height: room_h, sectors: 20 }),
            trim_mat
        );
        pillar1.transform.position = [room_w/2.0 - 1.2, room_h/2.0, -room_d/2.0 + 2.5].into();
        pillar1.set_body_type(BodyType::Static);
        self.add_object(pillar1);

        let mut pillar2 = GameObject::new(
            "Pillar_FrontLeft",
            Box::new(Cylinder { bottom_radius: 0.3, top_radius: 0.28, height: room_h, sectors: 20 }),
            trim_mat
        );
        pillar2.transform.position = [-room_w/2.0 + 1.2, room_h/2.0, -room_d/2.0 + 2.5].into();
        pillar2.set_body_type(BodyType::Static);
        self.add_object(pillar2);

        // 灯光
        let mut main_light = LightObject { name: "Main".to_string(), light: Light::POINT };
        main_light.light.position = [0.0, 5.0, 0.0];
        main_light.light.intensity = 1.8;
        main_light.light.color = [1.0, 0.95, 0.9];
        self.add_light(main_light);

        let mut target_light = LightObject { name: "TargetL".to_string(), light: Light::POINT };
        target_light.light.position = [0.0, 4.0, 8.0];
        target_light.light.intensity = 1.5;
        self.add_light(target_light);

        let mut sun = LightObject { name: "Sun".to_string(), light: Light::DERECTIONAL };
        sun.light.direction = [-0.5, -0.8, -0.3];
        sun.light.intensity = 0.6;
        self.add_light(sun);
        self.new_ambient_light("Ambient");

        // 玩家
        let aspect = self.default_aspect;
        self.new_camera("Player", aspect);
        if let Some(cam_idx) = self.get_selected_camera() {
            let cam = &mut self.cameras[cam_idx].camera;
            cam.transform.position = [0.0, 1.7, -18.0].into();
            cam.transform.look_at([0.0, 1.7, 0.0].into(), [0.0, 1.0, 0.0].into());
            cam.move_state = crate::scene::camera::MoveState::RigidBody;
            
            // 加载武器模型
            if let Ok(weapon_mesh) = crate::geometry::shape::mesh::Mesh::load_obj("assets/models/weapons/rifle.obj") {
                cam.weapon_mesh = Some(weapon_mesh);
                println!("✅ 武器模型加载成功");
            } else {
                println!("⚠️  武器模型加载失败");
            }
        }

        // 靶子
        self.spawn_target_in_house();
    }

    // 靶子生成逻辑 
    pub fn spawn_target_in_house(&mut self) {
        use rand::Rng; 
        let mut rng = rand::thread_rng();

        let x = rng.gen_range(-4.0..4.0);
        let y = rng.gen_range(1.0..3.5);
        let z = rng.gen_range(3.0..5.0); // 靠近后墙

        let target_mat = Material { ka: [0.8, 0.0, 0.0], kd: [1.0, 0.0, 0.0], ..Material::default() }; // 红色靶子

        let mut target = GameObject::new(
            "Target_Sphere", 
            Box::new(Sphere { radius: 0.4, col_divisions: 20, row_divisions: 20 }),
            target_mat,
        );
        target.transform.position = [x, y, z].into();
        target.set_body_type(BodyType::Static);
        
        self.add_object(target);
    }

    pub fn init_aimlab_scene(&mut self, _display: &glium::Display<glutin::surface::WindowSurface>) {
        // 清理旧场景
        self.objects.clear();
        self.lights.clear();
        self.cameras.clear();

        // 设置相机 (FPS 视角)
        let aspect = self.default_aspect;
        self.new_camera("MainCamera", aspect);
        if let Some(cam_idx) = self.get_selected_camera() {
            let cam = &mut self.cameras[cam_idx].camera;
            cam.transform.position = glam::Vec3::new(0.0, 1.7, 5.0); 
            cam.transform.look_at(glam::Vec3::ZERO, glam::Vec3::Y);
            cam.move_state = crate::scene::camera::MoveState::Free; 
        }

        // 顶部聚光灯
        let mut spot = LightObject {
            name: "CeilingLight".to_string(),
            light: Light::SPOT,
        };
        spot.light.position = [0.0, 10.0, 0.0];
        spot.light.direction = [0.0, -1.0, 0.0]; 
        spot.light.angle = 45.0;
        spot.light.range = 50.0;
        spot.light.intensity = 1.5;
        self.add_light(spot);

        // 微弱环境光，防止死黑
        let mut ambient = LightObject {
            name: "Ambient".to_string(),
            light: Light::AMBIENT,
        };
        ambient.light.intensity = 0.3; 
        self.add_light(ambient);

        let floor_mat = Material {
            ka: [0.2, 0.2, 0.2], kd: [0.3, 0.3, 0.3], ks: [0.1, 0.1, 0.1], ns: 10.0, ..Material::default()
        };
        let mut floor = GameObject::new(
            "Floor",
            Box::new(Cube { width: 30.0, height: 1.0, depth: 30.0 }),
            floor_mat,
        );
        floor.transform.position = [0.0, -0.5, 0.0].into(); 
        floor.set_body_type(BodyType::Static);
        self.add_object(floor);

        // 前墙
        let wall_mat = Material {
            ka: [0.5, 0.5, 0.5], kd: [0.6, 0.6, 0.6], ks: [0.1, 0.1, 0.1], ns: 10.0, ..Material::default()
        };
        let mut wall_front = GameObject::new(
            "Wall_Front",
            Box::new(Cube { width: 30.0, height: 10.0, depth: 1.0 }),
            wall_mat,
        );
        wall_front.transform.position = [0.0, 4.5, -10.0].into(); 
        wall_front.set_body_type(BodyType::Static);
        self.add_object(wall_front);

        // 生成初始靶子
        self.spawn_target();
    }

    // 随机生成靶子
    pub fn spawn_target(&mut self) {
        use rand::Rng; 
        let mut rng = rand::thread_rng();

        let x = rng.gen_range(-5.0..5.0);
        let y = rng.gen_range(1.0..4.0);
        
        let target_mat = Material {
            ka: [0.0, 0.8, 0.8], 
            kd: [0.0, 1.0, 1.0], 
            ks: [0.8, 0.8, 0.8], 
            ns: 64.0, 
            ..Material::default()
        };

        let mut target = GameObject::new(
            "Target_Sphere", 
            Box::new(Sphere { radius: 0.5, col_divisions: 32, row_divisions: 32 }),
            target_mat,
        );

        target.transform.position = [x, y, -9.0].into();
        target.set_body_type(BodyType::Static); 
        
        println!("New target spawned at: {:?}", target.transform.position);
        self.add_object(target);
    }

    pub fn handle_shoot(&mut self) {
        if let Some(cam_idx) = self.get_selected_camera() {
            let camera = &self.cameras[cam_idx].camera;
            let origin = camera.transform.position; 
            let forward = camera.transform.get_forward(); 

            println!("Bang! Shot fired from {:?} dir {:?}", origin, forward);

            let mut hit_idx = None;
            let mut min_dist = f32::MAX;

            // 遍历所有物体进行检测
            for (i, obj) in self.objects.iter().enumerate() {
                if !obj.name.starts_with("Target") {
                    continue;
                }
                
                let radius = 0.5 * obj.transform.scale.x; 
                let center = obj.transform.position;

                if let Some(dist) = ray_intersect_sphere(origin, forward, center, radius) {
                    if dist < min_dist {
                        min_dist = dist;
                        hit_idx = Some(i);
                    }
                }
            }

            // 处理命中结果
            if let Some(idx) = hit_idx {
                println!("Hit Target! Distance: {:.2}", min_dist);
                
                // 移除旧靶子
                self.objects.remove(idx);

                if self.selected_index == Some(idx) {
                    self.selected_index = None;
                } else if let Some(sel) = self.selected_index {
                    if sel > idx { self.selected_index = Some(sel - 1); }
                }

                // 生成新靶子
                // self.spawn_target();
                self.spawn_target_in_house();

            } else {
                println!("Miss!");
            }
        }
    }
}

fn ray_intersect_sphere(origin: glam::Vec3, dir: glam::Vec3, center: glam::Vec3, radius: f32) -> Option<f32> {
    let l = center - origin;
    let tca = l.dot(dir);
    if tca < 0.0 { return None; } 
    
    let d2 = l.dot(l) - tca * tca;
    let radius2 = radius * radius;
    if d2 > radius2 { return None; } 
    
    let thc = (radius2 - d2).sqrt();
    let t0 = tca - thc;
    let t1 = tca + thc;

    if t0 < 0.0 && t1 < 0.0 { return None; }
    
    if t0 < 0.0 { Some(t1) } else { Some(t0) }
}