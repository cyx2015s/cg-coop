use std::any::{Any, TypeId};

use crate::scene::world::{BodyType, EditableMesh, GameObject};
use crate::ui::UIBuild;
use gltf::Mesh;
use imgui::{Condition, Drag};

impl UIBuild for GameObject {
    fn build_ui(&mut self, ui: &imgui::Ui) {
        ui.window("属性面板 (Inspector)")
            .size([250.0, 500.0], Condition::FirstUseEver)
            .position([200.0, 0.0], Condition::FirstUseEver)
            .build(|| {
                ui.text_colored([0.0, 1.0, 0.0, 1.0], format!("当前选中: {}", self.name));
                ui.separator();
                ui.text("变换 (Transform)");
                let mut pos = self.transform.position.to_array();
                if Drag::new("位置").speed(0.1).build_array(ui, &mut pos) {
                    self.transform.position = pos.into();
                }
                let mut vel = self.physics.velocity;
                if Drag::new("速度").speed(0.1).build_array(ui, &mut vel) {
                    self.physics.velocity = vel.into();
                }
                let mut scale = self.transform.scale.to_array();
                if Drag::new("缩放").speed(0.01).build_array(ui, &mut scale) {
                    self.transform.scale = scale.into();
                }
                if ui.button("重置旋转") {
                    self.transform.rotation = glam::f32::Quat::IDENTITY;
                }

                ui.separator();

                let need_regen = self.shape.ui(ui);

                if need_regen {
                    self.regenerate_mesh();
                }

                // if self.kind != ShapeKind::Imported {

                if self.shape.intermediate_mesh()
                    && ui.button("网格化（不可逆！）")
                {
                    *self = GameObject {
                        name: self.name.clone() + " (Meshed)",
                        shape: Box::new(self.mesh.clone()),
                        mesh: self.mesh.clone(),
                        rendering: self.rendering.clone(),
                        physics: self.physics.clone(),
                        transform: self.transform.clone(),
                        behavior: self.behavior.clone(),
                    };
                }

                match self.rendering.selected_vertex_index {
                    Some(idx) => {
                        ui.text_colored([1.0, 1.0, 0.0, 1.0], format!("编辑顶点 {}", idx));
                        let v = &mut self.mesh.vertices[idx];
                        Drag::new("X").speed(0.01).build(ui, &mut v[0]);
                        Drag::new("Y").speed(0.01).build(ui, &mut v[1]);
                        Drag::new("Z").speed(0.01).build(ui, &mut v[2]);

                        let t = &mut self.mesh.tex_coords[idx];
                        Drag::new("U").speed(0.01).build(ui, &mut t[0]);
                        if Drag::new("V").speed(0.01).build(ui, &mut t[1]) {}
                    }
                    None => {
                        ui.text("未选中顶点");
                    }
                }

                ui.separator();
                ui.checkbox("显示/隐藏", &mut self.rendering.visible);
                ui.checkbox("启用纹理贴图", &mut self.rendering.use_texture);
                if ui.button("保存当前模型") {
                    let _ = self.mesh.save_obj("output.obj");
                }
                ui.separator();

                ui.text("物理属性(Physics)");

                let is_dynamic = self.physics.body_type == BodyType::Dynamic;
                if ui.radio_button_bool("自由物体(Dynamic}", is_dynamic) {
                    self.set_body_type(BodyType::Dynamic);
                }
                if ui.radio_button_bool("静态物体(Static)", !is_dynamic) {
                    self.set_body_type(BodyType::Static);
                }

                if self.physics.body_type == BodyType::Dynamic {
                    Drag::new("弹性系数")
                        .speed(0.01)
                        .range(0.0, 1.0)
                        .build(ui, &mut self.physics.restitution);
                }
            });
    }
}
