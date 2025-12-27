use super::camera::{Camera, MouseState};
use super::light::{AmbientLight, DirectionalLight, Light, PointLight, SpotLight};

use crate::core::material::{Material, Phong};
use crate::core::math::aabb::AABB;
use crate::core::math::transform::Transform;
use crate::geometry::shape::mesh::{AsMesh, Mesh};
use crate::geometry::shape::{cube::Cube, sphere::Sphere};
use crate::physics::collision::{apply_gravity, predict_position, resolve_collision};

use glutin::surface::WindowSurface;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyType {
    Static,  // 固定物体
    Dynamic, // 自由物体
}

// 交互行为枚举
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionBehavior {
    None,
    Door {
        is_open: bool,
        base_yaw: f32, // 初始角度
    },
    Window {
        is_broken: bool,
    },
}

#[derive(Clone, PartialEq)]
pub enum ShapeKind {
    Cube,
    Sphere,
    Cylinder,
    Cone,
    Imported,
    Nurbs,
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
            behavior: InteractionBehavior::None,
        };
        obj.regenerate_mesh();
        obj
    }

    // 计算物体的世界坐标 AABB
    pub fn aabb(&self) -> AABB {
        // 获取 Mesh 的局部包围盒的一半大小
        let half_size = (self.mesh.aabb.max - self.mesh.aabb.min) / 2.0;
        // 考虑缩放 (绝对值防止负缩放)
        let scaled_half_size = half_size * self.transform.scale.abs();
        
        // 简单的中心点 AABB 计算 
        AABB {
            min: self.transform.position - scaled_half_size,
            max: self.transform.position + scaled_half_size,
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
            default_mat: Phong::new([1.0, 0.5, 0.31], [1.0, 0.5, 0.31], [0.5, 0.5, 0.5], 32.0)
                .to_material(),
            debug: false,
            debug_frustum: false,
            layer: 0,
            gravity: [0.0, -9.8, 0.0],
        }
    }

    // 物理步进
    pub fn step(&mut self, dt: f32) {
        // 1. 处理门的动画 (插值旋转)
        for obj in &mut self.objects {
            if let InteractionBehavior::Door { is_open, base_yaw } = obj.behavior {
                // 如果开了，目标是转 90 度 (1.57弧度)
                let target_yaw = if is_open { base_yaw + 1.5708 } else { base_yaw };
                let current_rot = obj.transform.rotation;
                let target_rot = glam::f32::Quat::from_rotation_y(target_yaw);
                // 平滑过渡
                obj.transform.rotation = current_rot.slerp(target_rot, dt * 5.0);
            }
        }

        // 2. 物理模拟 
        let ptr = self.objects.as_mut_ptr();
        let len = self.objects.len();

        for i in 0..len {
            let dynamic_obj = unsafe { &mut *ptr.add(i) };

            // 只有可见且动态的物体才计算
            if !dynamic_obj.rendering.visible || dynamic_obj.physics.body_type != BodyType::Dynamic {
                continue;
            }

            apply_gravity(
                dynamic_obj,
                glam::f32::Vec3::from_array(self.gravity),
                dt,
            );

            let predicted_pos = predict_position(dynamic_obj, dt);

            let mut collided = false;
            for j in 0..len {
                if i == j { continue; }
                let static_obj = unsafe { &*ptr.add(j) };
                
                if !static_obj.rendering.visible { continue; }

                let static_aabb = static_obj.aabb(); 
                
                if resolve_collision(dynamic_obj, &static_aabb, predicted_pos) {
                    collided = true;
                }
            }
            if !collided {
                dynamic_obj.transform.position = predicted_pos;
            }
        }
    }

    // 交互输入逻辑
    pub fn handle_interaction_input(&mut self, player_pos: glam::f32::Vec3) {
        let mut nearest_idx = None;
        let mut min_dist = 3.0; // 3米内可交互

        for (i, obj) in self.objects.iter().enumerate() {
            if !obj.rendering.visible { continue; }
            let dist = obj.transform.position.distance(player_pos);
            if dist < min_dist {
                min_dist = dist;
                nearest_idx = Some(i);
            }
        }

        if let Some(idx) = nearest_idx {
            let behavior = self.objects[idx].behavior; // Copy enum
            match behavior {
                InteractionBehavior::Door { is_open, base_yaw } => {
                    // 切换状态
                    self.objects[idx].behavior = InteractionBehavior::Door { 
                        is_open: !is_open, 
                        base_yaw 
                    };
                }
                InteractionBehavior::Window { is_broken } => {
                    if !is_broken {
                        self.break_window(idx);
                    }
                }
                _ => {}
            }
        }
    }

    // 碎窗实现
    fn break_window(&mut self, window_idx: usize) {
        let window = &mut self.objects[window_idx];
        window.rendering.visible = false; // 隐藏原窗户
        // 标记为已碎
        if let InteractionBehavior::Window { is_broken } = &mut window.behavior {
            *is_broken = true;
        }

        // 生成碎片
        let pos = window.transform.position;
        let size = 1.0; 
        let rows = 4;
        let cols = 4;
        let step = size / 4.0;
        let mut shards = Vec::new();
        
        for r in 0..rows {
            for c in 0..cols {
                let offset_x = (c as f32 * step) - size/2.0 + step/2.0;
                let offset_y = (r as f32 * step) - size/2.0 + step/2.0;
                
                let mut shard = GameObject::new(
                    "Shard",
                    Box::new(Cube { width: step*0.9, height: step*0.9, depth: 0.05 }), 
                    window.rendering.material
                );
                shard.transform.position = pos + glam::vec3(offset_x, offset_y, 0.0);
                shard.physics.body_type = BodyType::Dynamic;
                // 给个随机速度炸开
                let rx = (c as f32 - 1.5) * 2.0;
                let ry = (r as f32 - 1.5) * 2.0;
                shard.physics.velocity = [rx, ry, 5.0];
                shards.push(shard);
            }
        }
        self.objects.append(&mut shards);
    }

    // 创建门 helper
    pub fn create_door(&mut self, pos: glam::f32::Vec3) {
        let width = 1.0;
        let height = 2.0;
        
        let mut door = GameObject::new(
            "Door",
            Box::new(Cube { width, height, depth: 0.1 }),
            self.default_mat
        );
        
        // 偏移顶点，让轴心在门边
        let offset = glam::vec3(width / 2.0, 0.0, 0.0);
        for v in &mut door.mesh.vertices {
            v[0] += offset.x;
            v[1] += offset.y;
            v[2] += offset.z;
        }
        // 更新包围盒
        door.mesh.aabb = AABB::default();
        for v in &door.mesh.vertices {
            door.mesh.aabb.union_point_array(*v);
        }

        door.transform.position = pos;
        door.behavior = InteractionBehavior::Door { 
            is_open: false, 
            base_yaw: 0.0 
        };
        door.set_body_type(BodyType::Static);
        self.add_object(door);
    }

    // 创建窗 helper
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
        aabb
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
        if let Some(idx) = self.selected_camera
            && idx < self.cameras.len() {
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
            && idx < self.lights.len() {
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
            && idx < self.objects.len() {
                return Some(&mut self.objects[idx]);
            }
        None
    }
}
