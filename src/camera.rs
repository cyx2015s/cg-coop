use super::base::transform;

pub enum MoveState {
    Locked,
    Free,
    PanObit,
}

impl PartialEq for MoveState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MoveState::Locked, MoveState::Locked) => true,
            (MoveState::Free, MoveState::Free) => true,
            (MoveState::PanObit, MoveState::PanObit) => true,
            _ => false,
        }
    }
}

pub struct Camera {
    pub transform: transform::Transform,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
    pub pitch: f32,
    pub yaw: f32,

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
            fovy: 3.141592 / 3.0,
            aspect,
            znear: 0.1,
            zfar: 1024.0,
            pitch: 0.0,
            yaw: 0.0,

            move_state: MoveState::Locked,
            pan_obit_speed: 0.01,
            pan_obit_pitch: 0.0,
            pan_obit_angle: 0.0,
            pan_obit_radius: 1.0,
            pan_obit_center: [0.0, 0.0, 0.0],
        }
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        self.pitch = (self.pitch + pitch).clamp(-1.55, 1.55);
        self.yaw += yaw;
        let yaw_quat = glam::f32::Quat::from_axis_angle(glam::f32::Vec3::Y, self.yaw);
        let pitch_quat = glam::f32::Quat::from_axis_angle(glam::f32::Vec3::X, self.pitch);
        self.transform.rotation = yaw_quat * pitch_quat * glam::f32::Quat::IDENTITY;
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
        // println!("{:?}", self.transform.position + self.transform.get_forward() * 5.0);
    }

    pub fn stop_pan_obit(&mut self) {
        self.move_state = MoveState::Locked;
    }

    pub fn get_projection_matrix(&self) -> [[f32; 4]; 4] {
        glam::f32::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
            .to_cols_array_2d()
    }
}
