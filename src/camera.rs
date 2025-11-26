use super::base::transform;
pub struct Camera {
    pub transform: transform::Transform,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            transform: transform::Transform::new(),
            fovy: 3.141592 / 3.0,
            aspect: aspect,
            znear: 0.1,
            zfar: 1024.0,
        }
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        let yaw_quat = glam::f32::Quat::from_axis_angle(glam::f32::Vec3::Y, yaw);
        let pitch_quat = glam::f32::Quat::from_axis_angle(self.transform.get_right(), pitch);
        self.transform.rotation = yaw_quat * self.transform.rotation;
        self.transform.rotation = pitch_quat * self.transform.rotation;
    }

    // 获取视图矩阵
    pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
        let view_matrix = glam::f32::Mat4::look_to_rh(
            self.transform.position,      // 相机位置
            self.transform.get_forward(), // 目标点（相机位置 + 前向方向）
            self.transform.get_up(),      // 上方向
        )
        .to_cols_array_2d();
        view_matrix
    }

    pub fn get_projection_matrix(&self) -> [[f32; 4]; 4] {
        glam::f32::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar)
            .to_cols_array_2d()
    }
}
