use crate::scene::world::LightObject;
use crate::ui::UIBuild;
use imgui::{Condition, Drag};

impl UIBuild for LightObject {
    fn build_ui(&mut self, ui: &imgui::Ui) {
        ui.window("灯光面板 (Inspector)")
            .size([250.0, 500.0], Condition::FirstUseEver)
            .position([240.0, 150.0], Condition::FirstUseEver)
            .build(|| {
                ui.text_colored([0.0, 1.0, 0.0, 1.0], &format!("当前选中: {}", self.name));
                ui.separator();

                if self.light.light_type == 0 {
                    ui.slider("环境光强", 0.0, 1.0, &mut self.light.intensity);
                    ui.color_edit3("环境光颜色", &mut self.light.color);
                }
                if self.light.light_type == 1 {
                    ui.slider("平行光强", 0.0, 1.0, &mut self.light.intensity);
                    Drag::new("平行光位置")
                        .speed(0.1)
                        .build_array(ui, &mut self.light.position);
                    Drag::new("平行光方向")
                        .speed(0.1)
                        .build_array(ui, &mut self.light.direction);
                    ui.color_edit3("平行光颜色", &mut self.light.color);
                }
                if self.light.light_type == 2 {
                    ui.slider("点光强", 0.0, 1.0, &mut self.light.intensity);
                    Drag::new("点光位置")
                        .speed(0.1)
                        .build_array(ui, &mut self.light.position);
                    ui.color_edit3("点光颜色", &mut self.light.color);
                    Drag::new("kc")
                        .speed(0.1)
                        .build(ui, &mut self.light.kfactor[0]);
                    Drag::new("kl")
                        .speed(0.1)
                        .build(ui, &mut self.light.kfactor[1]);
                    Drag::new("kq")
                        .speed(0.1)
                        .build(ui, &mut self.light.kfactor[2]);
                }
                if self.light.light_type == 3 {
                    ui.slider("聚光强", 0.0, 1.0, &mut self.light.intensity);
                    Drag::new("聚光位置")
                        .speed(0.1)
                        .build_array(ui, &mut self.light.position);
                    Drag::new("聚光方向")
                        .speed(0.1)
                        .build_array(ui, &mut self.light.direction);
                    ui.color_edit3("聚光颜色", &mut self.light.color);
                    Drag::new("聚光角度")
                        .speed(0.1)
                        .build(ui, &mut self.light.angle);
                    Drag::new("kc")
                        .speed(0.1)
                        .build(ui, &mut self.light.kfactor[0]);
                    Drag::new("kl")
                        .speed(0.1)
                        .build(ui, &mut self.light.kfactor[1]);
                    Drag::new("kq")
                        .speed(0.1)
                        .build(ui, &mut self.light.kfactor[2]);
                }
            });
    }
}
