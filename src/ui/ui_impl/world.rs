use crate::core::math::ray;
use crate::geometry::shape::mesh::Mesh;
use crate::scene::camera;
use crate::scene::world::{GameObject, ShapeKind, World};
use crate::ui::{UIBuild, UIHandle};
use imgui::Condition;

use glutin::surface::WindowSurface;
use std::time::Instant;

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
                        [0.0, 0.0, 0.0],
                        [0.0, -1.0, 0.0],
                        1.0,
                        [1.0, 1.0, 1.0],
                    );
                }
                ui.same_line();
                if ui.button("点光源") {
                    self.new_point_light(
                        &("Point Light".to_owned() + self.lights.len().to_string().as_str()),
                        [0.0, 0.0, 0.0],
                        1.0,
                        [1.0, 1.0, 1.0],
                        1.0,
                        0.09,
                        0.032,
                    );
                }
                ui.same_line();
                if ui.button("聚光灯") {
                    self.new_spot_light(
                        &("Spot Light".to_owned() + self.lights.len().to_string().as_str()),
                        [0.0, 0.0, 0.0],
                        [0.0, -1.0, 0.0],
                        1.0,
                        [1.0, 1.0, 1.0],
                        1.0,
                        0.09,
                        0.032,
                        12.5,
                    );
                }

                ui.separator();

                ui.text("基础形状:");
                if ui.button("立方体") {
                    self.add_object(GameObject::new(
                        "Cube",
                        ShapeKind::Cube {
                            width: 1.0,
                            height: 1.0,
                            depth: 1.0,
                        },
                        self.default_mat,
                    ));
                }
                ui.same_line();
                if ui.button("球体") {
                    self.add_object(GameObject::new(
                        "Sphere",
                        ShapeKind::Sphere {
                            radius: 0.5,
                            sectors: 32,
                        },
                        self.default_mat,
                    ));
                }

                ui.text("柱体/台体:");
                if ui.button("圆柱") {
                    self.add_object(GameObject::new(
                        "Cylinder",
                        ShapeKind::Cylinder {
                            top_radius: 0.5,
                            bottom_radius: 0.5,
                            height: 1.0,
                            sectors: 32,
                        },
                        self.default_mat,
                    ));
                }
                if ui.button("棱台") {
                    self.add_object(GameObject::new(
                        "Frustum",
                        ShapeKind::Cylinder {
                            top_radius: 0.3,
                            bottom_radius: 0.8,
                            height: 1.0,
                            sectors: 32,
                        },
                        self.default_mat,
                    ));
                }
                if ui.button("圆锥") {
                    self.add_object(GameObject::new(
                        "Cone",
                        ShapeKind::Cone {
                            radius: 0.5,
                            height: 1.0,
                            sectors: 32,
                        },
                        self.default_mat,
                    ));
                }

                ui.separator();
                ui.text("高级建模:");
                if ui.button("NURBS 曲面") {
                    let pts = vec![
                        [-1.5, 0.0, -1.5],
                        [-0.5, 0.5, -1.5],
                        [0.5, 0.5, -1.5],
                        [1.5, 0.0, -1.5],
                        [-1.5, 0.5, -0.5],
                        [-0.5, 1.5, -0.5],
                        [0.5, 1.5, -0.5],
                        [1.5, 0.5, -0.5],
                        [-1.5, 0.5, 0.5],
                        [-0.5, 1.5, 0.5],
                        [0.5, 1.5, 0.5],
                        [1.5, 0.5, 0.5],
                        [-1.5, 0.0, 1.5],
                        [-0.5, 0.5, 1.5],
                        [0.5, 0.5, 1.5],
                        [1.5, 0.0, 1.5],
                    ];
                    let weights = vec![1.0; 16];
                    self.add_object(GameObject::new(
                        "Nurbs Surface",
                        ShapeKind::Nurbs {
                            degree: 3,
                            control_points: pts,
                            weights,
                            u_count: 4,
                            v_count: 4,
                            current_nurbs_idx: 0,
                        },
                        self.default_mat,
                    ));
                }
                if ui.button("导入模型") {
                    if let Ok(mesh) = Mesh::load_obj("output.obj") {
                        let mut obj =
                            GameObject::new("Imported", ShapeKind::Imported, self.default_mat);
                        obj.mesh = mesh;
                        self.add_object(obj);
                    }
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
        let mut mouse_click_center = None;
        let mut mouse_click_inf = None;
        if let Some(idx) = self.get_selected_camera() {
            let camera = &mut self.cameras[idx].camera;
            let current_time = Instant::now();
            let delta_time = current_time
                .duration_since(self.last_frame_time)
                .as_secs_f32();
            self.last_frame_time = current_time;
            // 相机视场
            {
                let scroll = ui.io().mouse_wheel;
                camera.fovy *= f32::exp(-scroll * 0.005);
                camera.fovy = camera.fovy.clamp(0.05, 1.5);
            }

            // 相机移动控制
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
            }

            // 其余控制键
            {
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
                    let image: glium::texture::RawImage2d<'_, u8> =
                        display.read_front_buffer().unwrap();
                    let image = image::ImageBuffer::from_raw(
                        image.width,
                        image.height,
                        image.data.into_owned(),
                    )
                    .unwrap();
                    let image = image::DynamicImage::ImageRgba8(image).flipv();
                    image.save("screenshot.png").unwrap();
                }
            }

            if ui.is_mouse_clicked(imgui::MouseButton::Left)
                && !ui.is_any_item_focused()
                && !ui.is_any_item_hovered()
            {
                // 计算射线
                let mouse_pos = ui.io().mouse_pos;
                let [win_w, win_h] = ui.io().display_size;
                let ndc_x = (2.0 * mouse_pos[0]) / win_w - 1.0;
                let ndc_y: f32 = 1.0 - (2.0 * mouse_pos[1]) / win_h;
                let ndc_pos = glam::Vec3::new(ndc_x, ndc_y, 1.0);
                let inv_proj =
                    glam::Mat4::from_cols_array_2d(&camera.get_projection_matrix()).inverse();
                let inv_view = glam::Mat4::from_cols_array_2d(&camera.get_view_matrix()).inverse();
                let ray_dir_camera = (inv_proj * ndc_pos.extend(1.0)).truncate().normalize();
                mouse_click_inf = Some(
                    (inv_view * ray_dir_camera.extend(0.0))
                        .truncate()
                        .normalize(),
                );
                mouse_click_center = Some(
                    (inv_view * glam::Vec3::from(camera.get_position()).extend(1.0)).truncate(),
                );
            }

            if camera.move_state == camera::MoveState::PanObit {
                camera.update_pan_obit(delta_time);
            }
        }

        // 顶点选择
        if let (Some(mouse_click_center), Some(mouse_click_inf)) =
            (mouse_click_center, mouse_click_inf)
        {
            if let Some(game_obj) = self.get_selected_mut() {
                if (game_obj.kind == ShapeKind::Imported) {
                    let inv_model = game_obj.transform.get_matrix().inverse();
                    let local_mouse_click_center = inv_model * mouse_click_center.extend(1.0);
                    let local_mouse_click_inf = inv_model * mouse_click_inf.extend(0.0);
                    let local_ray_o = local_mouse_click_center.truncate();
                    let local_ray_d = (local_mouse_click_inf.truncate() - local_ray_o).normalize();
                    let camera_ray = ray::Ray {
                        o: local_ray_o,
                        d: local_ray_d,
                        tMax: f32::INFINITY,
                    };
                    match game_obj
                        .mesh
                        .compute_closest_point(camera_ray.o.to_array(), camera_ray.d.to_array())
                    {
                        Some((pt, cos_angle)) => {
                            // 找到最近点，标记选中
                            if cos_angle < 0.99 {
                                game_obj.selected_vertex_index = None;
                                return;
                            }
                            for (i, v) in game_obj.mesh.vertices.iter().enumerate() {
                                let v_pos = glam::Vec3::from(*v);
                                if (v_pos - glam::Vec3::from(pt)).length() < 0.1 {
                                    game_obj.selected_vertex_index = Some(i);
                                    println!("selected point: {:?}", pt);
                                    break;
                                }
                            }
                        }
                        None => {
                            game_obj.selected_vertex_index = None;
                            println!("no point found");
                        }
                    }
                }
            }
        }
    }
}
