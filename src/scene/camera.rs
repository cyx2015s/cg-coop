use std::f32;

use crate::{
    core::math::transform,
    scene::world::{BodyType, PhysicalProperties},
};

pub enum MoveState {
    Locked,
    RigidBody,
    Free,
    PanObit,
}

impl PartialEq for MoveState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MoveState::Locked, MoveState::Locked) => true,
            (MoveState::RigidBody, MoveState::RigidBody) => true,
            (MoveState::Free, MoveState::Free) => true,
            (MoveState::PanObit, MoveState::PanObit) => true,
            _ => false,
        }
    }
}

pub struct Camera {
    pub transform: transform::Transform,
    pub physics: PhysicalProperties,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub force: f32,
    pub up_velocity: f32,

    pub move_state: MoveState,
    pub pan_obit_speed: f32,
    pub pan_obit_pitch: f32,
    pub pan_obit_angle: f32,
    pub pan_obit_radius: f32,
    pub pan_obit_center: [f32; 3],
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            transform: transform::Transform::default(),
            physics: PhysicalProperties::default(),
            fovy: f32::consts::PI / 3.0,
            aspect,
            znear: 0.1,
            zfar: 50.0,
            pitch: 0.0,
            yaw: 0.0,
            force: 9.5,
            up_velocity: 4.0,

            move_state: MoveState::Locked,
            pan_obit_speed: 1.0,
            pan_obit_pitch: 0.0,
            pan_obit_angle: 0.0,
            pan_obit_radius: 1.0,
            pan_obit_center: [0.0, 0.0, 0.0],
        }
    }

    pub fn init(&mut self) {
        self.transform.position = [0.0, 0.0, 10.0].into();
        self.transform
            .look_at([0.0, 0.0, 0.0].into(), [0.0, 1.0, 0.0].into());
        self.physics.friction = [8.0; 3];
        self.rotate(0.0, 0.0);
    }

    pub fn get_position(&self) -> [f32; 3] {
        self.transform.position.to_array()
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        self.pitch = (self.pitch + pitch).clamp(-1.55, 1.55);
        self.yaw += yaw;
        let yaw_quat = glam::f32::Quat::from_axis_angle(glam::f32::Vec3::Y, self.yaw);
        let pitch_quat = glam::f32::Quat::from_axis_angle(glam::f32::Vec3::X, self.pitch);
        self.transform.rotation = yaw_quat * pitch_quat * glam::f32::Quat::IDENTITY;
    }

    pub fn set_dynamic(&mut self) {
        self.physics.body_type = BodyType::Dynamic;
    }

    pub fn set_static(&mut self) {
        self.physics.body_type = BodyType::Static;
    }
    // 获取视图矩阵
    pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        glam::f32::Mat4::look_to_rh(
            self.transform.position,      // 相机位置
            self.transform.get_forward(), // 目标点（相机位置 + 前向方向）
            glam::f32::Vec3::Y,           // 上方向
        )
        .to_cols_array_2d()
    }

    pub fn get_view_no_translation(&self) -> [[f32; 4]; 4] {
        glam::f32::Mat4::look_to_rh(
            glam::f32::Vec3::ZERO,
            self.transform.get_forward(),
            glam::f32::Vec3::Y,
        )
        .to_cols_array_2d()
    }

    pub fn start_pan_obit(&mut self, angle: f32, raduis: f32, center: [f32; 3]) {
        self.move_state = MoveState::PanObit;
        self.pan_obit_pitch = std::f32::consts::PI * angle / 180.0;
        self.pan_obit_angle = 0.0;
        self.pan_obit_radius = raduis;
        self.pan_obit_center = center;
    }

    pub fn update_pan_obit(&mut self, delta_time: f32) {
        self.pan_obit_angle += delta_time * self.pan_obit_speed;
        self.transform.position.y = self.pan_obit_pitch.sin() * self.pan_obit_radius;
        self.transform.position.x =
            self.pan_obit_pitch.cos() * self.pan_obit_radius * self.pan_obit_angle.sin();
        self.transform.position.z =
            self.pan_obit_pitch.cos() * self.pan_obit_radius * self.pan_obit_angle.cos();
        self.transform.position += glam::f32::Vec3::from(self.pan_obit_center);
        self.transform.look_at(
            glam::f32::Vec3::from(self.pan_obit_center),
            glam::f32::Vec3::NEG_Y,
        );
        // println!("{:?}", self.transform.position);
        // println!("{:?}", self.transform.position + self.transform.get_forward() * 5.0);
    }

    pub fn stop_pan_obit(&mut self) {
        self.move_state = MoveState::Locked;
    }

    pub fn get_projection_matrix(&self) -> [[f32; 4]; 4] {
        glam::f32::Mat4::perspective_rh_gl(self.fovy, self.aspect, self.znear, self.zfar)
            .to_cols_array_2d()
    }

    pub fn update_impluse(&mut self, flag: [bool; 6]) {
        let forward_origin = self.transform.get_forward();
        let forward = glam::f32::Vec3::new(forward_origin.x, 0.0, forward_origin.z).normalize();
        let up = glam::f32::Vec3::Y;
        let right = up.cross(forward).normalize();
        let mut f_force = forward * self.force;
        let mut r_force = right * self.force;
        if flag[0] != flag[1] && !flag[0] {
            f_force *= -1.0;
        } else if flag[0] == flag[1] {
            f_force *= 0.0;
        }
        if flag[2] != flag[3] && !flag[2] {
            r_force *= -1.0;
        } else if flag[2] == flag[3] {
            r_force *= 0.0;
        }
        if flag[4] && self.physics.velocity[1].abs() < 0.01 {
            self.physics.velocity[1] = self.up_velocity;
        }
        self.physics.force = (f_force + r_force).to_array();
    }
}

#[derive(Debug)]
pub struct MouseState {
    pub is_locked: bool,
    pub is_visible: bool,
    pub sensitivity: f32,
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            is_locked: false,
            is_visible: true,
            sensitivity: 0.01,
        }
    }

    pub fn handle_mouse_move(
        &mut self,
        delta: (f64, f64),
        camera: &mut Camera,
        window: &glium::winit::window::Window,
    ) {
        if self.is_locked {
            let (dx, dy) = delta;
            camera.rotate(
                (-dx as f32) * self.sensitivity,
                (-dy as f32) * self.sensitivity,
            );
            window.request_redraw();
        }
    }

    pub fn toggle_lock(&mut self, window: &glium::winit::window::Window) -> bool {
        self.is_locked = !self.is_locked;
        self.is_visible = !self.is_visible;

        if self.is_locked {
            // 尝试锁定
            if window
                .set_cursor_grab(glium::winit::window::CursorGrabMode::Confined)
                .is_ok()
                || window
                    .set_cursor_grab(glium::winit::window::CursorGrabMode::Locked)
                    .is_ok()
            {
                window.set_cursor_visible(false);
                println!("鼠标已锁定");
                true
            } else {
                self.is_locked = false;
                println!("无法锁定鼠标");
                false
            }
        } else {
            // 释放
            let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
            println!("鼠标已释放");
            true
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }
}
