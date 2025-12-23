use crate::scene::world::{ GameObject, ShapeKind };
use imgui::{ Condition, Drag};
use crate::ui::UIBuild;

impl UIBuild for GameObject {
    fn build_ui(&mut self, ui: &imgui::Ui) {
        ui.window("属性面板 (Inspector)")
            .size([250.0, 500.0], Condition::FirstUseEver)
            .position([240.0, 150.0], Condition::FirstUseEver)
            .build(|| {
                ui.text_colored([0.0, 1.0, 0.0, 1.0], &format!("当前选中: {}", self.name));
                ui.separator();
                ui.text("变换 (Transform)");
                let mut pos = self.transform.position.to_array();
                if Drag::new("位置").speed(0.1).build_array(ui, &mut pos) { self.transform.position = pos.into(); }
                let mut scale = self.transform.scale.to_array();
                if Drag::new("缩放").speed(0.01).build_array(ui, &mut scale) { self.transform.scale = scale.into(); }
                if ui.button("重置旋转") { self.transform.rotation = glam::f32::Quat::IDENTITY; }

                ui.separator();
                ui.text("形状参数 (Parameters)");
                
                let mut need_regen = false;
                match &mut self.kind {
                    ShapeKind::Cube { width, height, depth } => {
                        if Drag::new("宽").speed(0.1).build(ui, width) { need_regen = true; }
                        if Drag::new("高").speed(0.1).build(ui, height) { need_regen = true; }
                        if Drag::new("深").speed(0.1).build(ui, depth) { need_regen = true; }
                    },
                    ShapeKind::Sphere { radius, sectors } => {
                        if Drag::new("半径").speed(0.05).build(ui, radius) { need_regen = true; }
                        let mut s = *sectors as i32;
                        if ui.slider( "精度", 3, 64, &mut s) { *sectors = s as u16; need_regen = true; }
                    },
                    ShapeKind::Cylinder { top_radius, bottom_radius, height, sectors } => {
                        if Drag::new("顶半径").speed(0.05).build(ui, top_radius) { need_regen = true; }
                        if Drag::new("底半径").speed(0.05).build(ui, bottom_radius) { need_regen = true; }
                        if Drag::new("高度").speed(0.1).build(ui, height) { need_regen = true; }
                        let mut s = *sectors as i32;
                        if ui.slider( "精度", 3, 64, &mut s) { *sectors = s as u16; need_regen = true; }
                    },
                    ShapeKind::Cone { radius, height, sectors } => {
                        if Drag::new("底半径").speed(0.05).build(ui, radius) { need_regen = true; }
                        if Drag::new("高度").speed(0.1).build(ui, height) { need_regen = true; }
                        let mut s = *sectors as i32;
                        if ui.slider( "精度", 3, 64, &mut s) { *sectors = s as u16; need_regen = true; }
                    },
                    ShapeKind::Nurbs { control_points, weights, current_nurbs_idx, .. } => {
                        ui.text("NURBS 控制点编辑");

                        // 1️⃣ 确保 idx 一定存在（没有就初始化为 0）
                        let idx = current_nurbs_idx;

                        // 2️⃣ ImGui Slider 直接绑定 usize
                        ui.slider( "点索引", 0, control_points.len().saturating_sub(1), idx);

                        if *idx < control_points.len() {
                            ui.text_colored([1.0, 1.0, 0.0, 1.0], "编辑中...");

                            if Drag::new("X")
                                .speed(0.05)
                                .build(ui, &mut control_points[*idx][0])
                            {
                                need_regen = true;
                            }

                            if Drag::new("Y")
                                .speed(0.05)
                                .build(ui, &mut control_points[*idx][1])
                            {
                                need_regen = true;
                            }

                            if Drag::new("Z")
                                .speed(0.05)
                                .build(ui, &mut control_points[*idx][2])
                            {
                                need_regen = true;
                            }

                            if Drag::new("权重")
                                .speed(0.05)
                                .range(0.1, 100.0)
                                .build(ui, &mut weights[*idx])
                            {
                                need_regen = true;
                            }
                        }
                    }

                    _ => {}
                }
                
                if need_regen {
                    self.regenerate_mesh();
                }

                if self.kind != ShapeKind::Imported {
                    if ui.button("网格化（不可逆！）") {
                        *self = GameObject {
                            name: self.name.clone() + " (Meshed)",
                            transform: self.transform.clone(),
                            material: self.material,
                            mesh: self.mesh.clone(),
                            kind: ShapeKind::Imported,
                            visible: self.visible,
                            use_texture: self.use_texture,
                            selected_vertex_index: None,
                        }
                        
                    }
                }  else {
                    ui.text("模型已网格化，只能编辑顶点位置和UV映射。");
                }
                match self.selected_vertex_index {
                    Some(idx) => {
                        ui.text_colored([1.0, 1.0, 0.0, 1.0], &format!("编辑顶点 {}", idx));
                        let v = &mut self.mesh.vertices[idx];
                        if Drag::new("X").speed(0.01).build(ui, &mut v[0]) {}
                        if Drag::new("Y").speed(0.01).build(ui, &mut v[1]) {}
                        if Drag::new("Z").speed(0.01).build(ui, &mut v[2]) {}
                        
                        let t = &mut self.mesh.tex_coords[idx];
                        if Drag::new("U").speed(0.01).build(ui, &mut t[0]) {}
                        if Drag::new("V").speed(0.01).build(ui, &mut t[1]) {}
                    },
                    None => {
                        ui.text("未选中顶点");
                    }
                }
                
                ui.separator();
                ui.checkbox("显示/隐藏", &mut self.visible);
                ui.checkbox("启用纹理贴图", &mut self.use_texture);
                if ui.button("保存当前模型") { let _ = self.mesh.save_obj("output.obj"); }
            });
    }
}
