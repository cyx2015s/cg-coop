use crate::geometry::shape::cone::Cone;
use crate::geometry::shape::cube::Cube;
use crate::geometry::shape::cylinder::Cylinder;
use crate::geometry::shape::mesh::Mesh;
use crate::geometry::shape::nurbs::NurbsSurface;
use crate::geometry::shape::sphere::Sphere;
use crate::scene::camera;
use crate::scene::world::{GameObject, World};
use crate::ui::{UIBuild, UIHandle};
use imgui::Condition;

use glutin::surface::WindowSurface;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

impl UIBuild for World {
    fn build_ui(&mut self, ui: &imgui::Ui) {
        ui.window("场景列表 (Scene List)")
            .size([200.0, 400.0], Condition::FirstUseEver)
            .position([20.0, 150.0], Condition::FirstUseEver)
            .build(|| {
                if ui.button("新建相机") {
                    self.new_camera(
                        &("Camera".to_owned() + self.cameras.len().to_string().as_str()),
                        self.default_aspect,
                    );
                }

                ui.separator();

                ui.text("灯光类型:");
                if ui.button("平行光") {
                    self.new_directional_light(
                        &("Directional Light".to_owned() + self.lights.len().to_string().as_str()),
                    );
                }
                ui.same_line();
                if ui.button("点光源") {
                    self.new_point_light(
                        &("Point Light".to_owned() + self.lights.len().to_string().as_str()),
                    );
                }
                ui.same_line();
                if ui.button("聚光灯") {
                    self.new_spot_light(
                        &("Spot Light".to_owned() + self.lights.len().to_string().as_str()),
                    );
                }

                ui.separator();

                ui.text("基础形状:");
                if ui.button("立方体") {
                    self.add_object(GameObject::new(
                        "Cube",
                        Box::new(Cube { width: 1.0, height: 1.0, depth: 1.0 }),
                        self.default_mat,
                    ));
                }
                ui.same_line();
                if ui.button("球体") {
                    self.add_object(GameObject::new(
                        "Sphere",
                        Box::new(Sphere { radius: 0.5, col_divisions: 32, row_divisions: 32 }),
                        self.default_mat,
                    ));
                }

                ui.text("柱体/台体:");
                if ui.button("圆柱") {
                    self.add_object(GameObject::new(
                        "Cylinder",
                        Box::new(Cylinder { top_radius: 0.5, bottom_radius: 0.5, height: 1.0, sectors: 32 }),
                        self.default_mat,
                    ));
                }
                if ui.button("棱台") {
                    self.add_object(GameObject::new(
                        "Frustum",
                        Box::new(Cylinder { top_radius: 0.3, bottom_radius: 0.8, height: 1.0, sectors: 32 }),
                        self.default_mat,
                    ));
                }
                if ui.button("圆锥") {
                    self.add_object(GameObject::new(
                        "Cone",
                        Box::new(Cone { radius: 0.5, height: 1.0, sectors: 32 }),
                        self.default_mat,
                    ));
                }

                ui.separator();
                ui.text("高级建模:");
                
                if ui.button("NURBS 球体") {
                    let r = 1.0; 
                    let w_corner = 0.70710678; 
                    let v_params = vec![
                        (0.0, -r, 1.0), (r, -r, w_corner), (r, 0.0, 1.0), (r, r, w_corner), (0.0, r, 1.0),
                    ];
                    let u_params = vec![
                        (1.0, 0.0, 1.0), (1.0, 1.0, w_corner), (0.0, 1.0, 1.0), (-1.0, 1.0, w_corner), 
                        (-1.0, 0.0, 1.0), (-1.0, -1.0, w_corner), (0.0, -1.0, 1.0), (1.0, -1.0, w_corner), (1.0, 0.0, 1.0),
                    ];
                    let mut control_points = Vec::new();
                    let mut weights = Vec::new();
                    for v_p in &v_params {
                        for u_p in &u_params {
                            control_points.push([u_p.0 * v_p.0, v_p.1, u_p.1 * v_p.0]);
                            weights.push(u_p.2 * v_p.2);
                        }
                    }
                    let u_knots = vec![0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0];
                    let v_knots = vec![0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0];

                    let mut obj = GameObject::new(
                        "NURBS Sphere",
                        Box::new(NurbsSurface {
                            degree: 2,
                            control_points,
                            weights,
                            u_count: 9,
                            v_count: 5,
                            splits: 32,
                            selected_point_idx: 0,
                            u_knots, 
                            v_knots, 
                        }),
                        self.default_mat,
                    );
                    obj.transform.position.y = 2.0;
                    self.add_object(obj);
                }

                if ui.button("NURBS 曲面 (默认)") {
                    let pts = vec![
                        [-1.5, 0.0, -1.5], [-0.5, 0.5, -1.5], [0.5, 0.5, -1.5], [1.5, 0.0, -1.5],
                        [-1.5, 0.5, -0.5], [-0.5, 1.5, -0.5], [0.5, 1.5, -0.5], [1.5, 0.5, -0.5],
                        [-1.5, 0.5, 0.5], [-0.5, 1.5, 0.5], [0.5, 1.5, 0.5], [1.5, 0.5, 0.5],
                        [-1.5, 0.0, 1.5], [-0.5, 0.5, 1.5], [0.5, 0.5, 1.5], [1.5, 0.0, 1.5],
                    ];
                    let weights = vec![1.0; 16];
                    self.add_object(GameObject::new(
                        "Nurbs Surface",
                        Box::new(NurbsSurface {
                            degree: 3,
                            control_points: pts,
                            weights,
                            u_count: 4,
                            v_count: 4,
                            splits: 32,
                            selected_point_idx: 0,
                            u_knots: vec![], 
                            v_knots: vec![], 
                        }),
                        self.default_mat,
                    ));
                }
                if ui.button("导入模型")
                    && let Ok(mesh) = Mesh::load_obj("output.obj") {
                        let obj = GameObject::new("Imported", Box::new(mesh), self.default_mat);
                        self.add_object(obj);
                    }

                ui.separator();
                ui.text("交互物体:");
                if ui.button("生成门 (按E开关)") {
                    self.create_door(glam::vec3(-2.0, 1.0, 0.0));
                }
                ui.same_line();
                if ui.button("生成窗 (按F击碎)") {
                    self.create_window(glam::vec3(2.0, 1.5, 0.0));
                }

                ui.separator();
                ui.text("场景物体:");
                for (i, obj) in self.objects.iter().enumerate() {
                    let is_selected = self.selected_index == Some(i);
                    if ui
                        .selectable_config(&format!("{}: {}", i, obj.name))
                        .selected(is_selected)
                        .build()
                    {
                        self.selected_index = Some(i);
                    }
                }
                ui.separator();
                ui.text("场景灯光:");
                for (i, obj) in self.lights.iter().enumerate() {
                    let is_selected = self.selected_light == Some(i);
                    if ui
                        .selectable_config(&format!("{}: {}", i, obj.name))
                        .selected(is_selected)
                        .build()
                    {
                        self.selected_light = Some(i);
                    }
                }

                ui.separator();
                ui.text("场景相机:");
                for (i, obj) in self.cameras.iter().enumerate() {
                    let is_selected = self.selected_camera == Some(i);
                    if ui
                        .selectable_config(&format!("{}: {}", i, obj.name))
                        .selected(is_selected)
                        .build()
                    {
                        self.selected_camera = Some(i);
                    }
                }
            });

        if let Some(idx) = self.get_selected_camera() {
            let obj = &mut self.cameras[idx];
            obj.build_ui(ui);
        }

        if let Some(obj) = self.get_selected_mut() {
            obj.build_ui(ui);
        }

        if let Some(obj) = self.get_selected_light() {
            obj.build_ui(ui);
        }

        ui.window("调试操作")
            .size([200.0, 400.0], Condition::FirstUseEver)
            .position([20.0, 150.0], Condition::FirstUseEver)
            .build(|| {
                let items = [
                    "layer 0", "layer 1", "layer 2", "layer 3", "layer 4", "layer 5", "layer 6",
                    "layer 7",
                ];
                ui.checkbox("检查阴影贴图", &mut self.debug);
                ui.combo_simple_string("Layer", &mut self.layer, &items);
                ui.checkbox("视锥体显示", &mut self.debug_frustum)
            });
    }
}

impl UIHandle for World {
    fn handle_ui_input(&mut self, ui: &mut imgui::Ui, display: &glium::Display<WindowSurface>) {
        let mut mouse_click_near = None;
        let mut mouse_click_far = None;
        let mut interact_pos = None;
        
        if let Some(idx) = self.get_selected_camera() {
            let pos = self.cameras[idx].camera.transform.position;
            if ui.is_key_pressed(imgui::Key::E) || ui.is_key_pressed(imgui::Key::F) {
                interact_pos = Some(pos);
            }
        }

        if let Some(pos) = interact_pos {
            self.handle_interaction_input(pos);
        }

        if let Some(idx) = self.get_selected_camera() {
            let camera = &mut self.cameras[idx].camera;
            let current_time = Instant::now();
            let delta_time = current_time
                .duration_since(self.last_frame_time)
                .as_secs_f32();
            self.last_frame_time = current_time;
            
            {
                let scroll = ui.io().mouse_wheel;
                camera.fovy *= f32::exp(-scroll * 0.005);
                camera.fovy = camera.fovy.clamp(0.05, 1.5);
            }

            {
                let move_speed = 5.0 * delta_time;

                if camera.move_state == camera::MoveState::Free {
                    if ui.is_key_down(imgui::Key::W) {
                        camera.transform.position += camera.transform.get_forward() * move_speed;
                    }
                    if ui.is_key_down(imgui::Key::S) {
                        camera.transform.position -= camera.transform.get_forward() * move_speed;
                    }
                    if ui.is_key_down(imgui::Key::A) {
                        camera.transform.position -= camera.transform.get_right() * move_speed;
                    }
                    if ui.is_key_down(imgui::Key::D) {
                        camera.transform.position += camera.transform.get_right() * move_speed;
                    }
                    if ui.is_key_down(imgui::Key::Space) {
                        camera.transform.position += glam::f32::Vec3::Y * move_speed;
                    }
                    if ui.is_key_down(imgui::Key::LeftCtrl) {
                        camera.transform.position -= glam::f32::Vec3::Y * move_speed;
                    }
                }

                if camera.move_state == camera::MoveState::RigidBody {
                    
                    camera.set_dynamic();
                    self.camera_force[0] = ui.is_key_down(imgui::Key::W);
                    self.camera_force[1] = ui.is_key_down(imgui::Key::S);
                    self.camera_force[2] = ui.is_key_down(imgui::Key::A);
                    self.camera_force[3] = ui.is_key_down(imgui::Key::D);
                    self.camera_force[4] = ui.is_key_down(imgui::Key::Space);
                    self.camera_force[5] = ui.is_key_down(imgui::Key::LeftShift);
                    if ui.is_key_down(imgui::Key::LeftShift) { camera.force = 3.0 * 12.0; }
                    else { camera.force = 12.0; }
                    camera.update_impluse(self.camera_force);
                } else {
                    camera.set_static();
                }
            }

            {
                if ui.is_key_pressed(imgui::Key::K) {
                    camera.move_state = if camera.move_state == camera::MoveState::RigidBody {
                        camera::MoveState::Locked
                    } else {
                        camera::MoveState::RigidBody
                    };
                }
                if ui.is_key_pressed(imgui::Key::V) {
                    camera.move_state = if camera.move_state == camera::MoveState::Free {
                        camera::MoveState::Locked
                    } else {
                        camera::MoveState::Free
                    };
                }
                if ui.is_key_pressed(imgui::Key::B) {
                    if camera.move_state == camera::MoveState::PanObit {
                        camera.stop_pan_obit();
                    } else {
                        camera.start_pan_obit(0.0, 5.0, [0.5, 0.5, 0.5]);
                    }
                }
                if ui.is_key_pressed(imgui::Key::R) {
                    camera.fovy = 3.141592 / 2.0;
                }
                if ui.is_key_pressed(imgui::Key::P) {
                    let now = SystemTime::now();
                    let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
                    let timestamp = duration.as_secs();
                    
                    // [修复] 这里显式指定了泛型参数，解决 E0282 错误
                    if let Ok(image) = display.read_front_buffer::<glium::texture::RawImage2d<'_, u8>>() {
                        let image = image::ImageBuffer::from_raw(
                            image.width,
                            image.height,
                            image.data.into_owned(),
                        )
                        .unwrap();
                        let image = image::DynamicImage::ImageRgba8(image).flipv();
                        let _ = image.save(
                            "screenshot_".to_owned()
                                + &timestamp.to_string()
                                + ".png",
                        );
                    }
                }
            }

            if ui.is_mouse_clicked(imgui::MouseButton::Left)
                && !ui.is_any_item_focused()
                && !ui.is_any_item_hovered()
            {
                let mouse_pos = ui.io().mouse_pos;
                let [win_w, win_h] = ui.io().display_size;
                let ndc_x = (2.0 * mouse_pos[0]) / win_w - 1.0;
                let ndc_y: f32 = 1.0 - (2.0 * mouse_pos[1]) / win_h;
                let ndc_far = glam::Vec3::new(ndc_x, ndc_y, 1.0);
                let ndc_near = glam::Vec3::new(ndc_x, ndc_y, -1.0);
                let inv_proj =
                    glam::Mat4::from_cols_array_2d(&camera.get_projection_matrix()).inverse();
                let inv_view = glam::Mat4::from_cols_array_2d(&camera.get_view_matrix()).inverse();
                let world_far = inv_view * inv_proj * ndc_far.extend(1.0);
                let world_near = inv_view * inv_proj * ndc_near.extend(1.0);
                let world_far = world_far.truncate() / world_far.w;
                let world_near = world_near.truncate() / world_near.w;
                mouse_click_far = Some(world_far);
                mouse_click_near = Some(world_near);
            }

            if camera.move_state == camera::MoveState::PanObit {
                camera.update_pan_obit(delta_time);
            }
        }
    }
}