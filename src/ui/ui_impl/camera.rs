use crate::scene::world::CameraObject;
use crate::ui::UIBuild;
use imgui::Condition;

impl UIBuild for CameraObject {
    fn build_ui(&mut self, ui: &imgui::Ui) {
        ui.window("相机面板 (Inspector)")
            .size([250.0, 500.0], Condition::FirstUseEver)
            .position([240.0, 150.0], Condition::FirstUseEver)
            .build(|| {
                ui.text_colored([0.0, 1.0, 0.0, 1.0], &format!("当前选中: {}", self.name));
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
            });
    }
}
