use crate::scene::world::CameraObject;
use crate::ui::UIBuild;
use gltf::camera;
use imgui::{Condition, Drag};

impl UIBuild for CameraObject {
    fn build_ui(&mut self, ui: &imgui::Ui) {
        ui.window("相机面板 (Inspector)")
            .size([250.0, 500.0], Condition::FirstUseEver)
            .position([240.0, 150.0], Condition::FirstUseEver)
            .build(|| {
                ui.text_colored([0.0, 1.0, 0.0, 1.0], format!("当前选中: {}", self.name));
                ui.separator();

                let epsilon = 0.001;
                ui.slider("相机FOV", 0.0, 1.5, &mut self.camera.fovy);
                ui.slider(
                    "相机近裁剪面",
                    0.01,
                    self.camera.zfar - epsilon,
                    &mut self.camera.znear,
                );
                ui.slider(
                    "相机远裁剪面",
                    self.camera.znear + epsilon,
                    100.0,
                    &mut self.camera.zfar,
                );
                let mut pos = self.camera.transform.position.to_array();
                if Drag::new("位置").speed(0.1).build_array(ui, &mut pos) {
                    self.camera.transform.position = pos.into();
                }
                let mut sensitivity = self.camera.sensitivity;
                if Drag::new("灵敏度").speed(0.01).build(ui, &mut sensitivity) {
                    self.camera.sensitivity = sensitivity;
                }
                ui.slider("force", 1.0, 100.0, &mut self.camera.force);
                ui.slider("up_vel", 0.0, 100.0, &mut self.camera.up_velocity);
                ui.slider(
                    "friction_0",
                    0.0,
                    10.0,
                    &mut self.camera.physics.friction,
                );

            });
    }
}
