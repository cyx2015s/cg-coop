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
                        Box::new(Cube {
                            width: 1.0,
                            height: 1.0,
                            depth: 1.0,
                        }),
                        self.default_mat,
                    ));
                }
                ui.same_line();
                if ui.button("球体") {
                    self.add_object(GameObject::new(
                        "Sphere",
                        Box::new(Sphere {
                            radius: 0.5,
                            col_divisions: 32,
                            row_divisions: 32,
                        }),
                        self.default_mat,
                    ));
                }

                ui.text("柱体/台体:");
                if ui.button("圆柱") {
                    self.add_object(GameObject::new(
                        "Cylinder",
                        Box::new(Cylinder {
                            top_radius: 0.5,
                            bottom_radius: 0.5,
                            height: 1.0,
                            sectors: 32,
                        }),
                        self.default_mat,
                    ));
                }
                if ui.button("棱台") {
                    self.add_object(GameObject::new(
                        "Frustum",
                        Box::new(Cylinder {
                            top_radius: 0.3,
                            bottom_radius: 0.8,
                            height: 1.0,
                            sectors: 32,
                        }),
                        self.default_mat,
                    ));
                }
                if ui.button("圆锥") {
                    self.add_object(GameObject::new(
                        "Cone",
                        Box::new(Cone {
                            radius: 0.5,
                            height: 1.0,
                            sectors: 32,
                        }),
                        self.default_mat,
                    ));
                }

                ui.separator();
                ui.text("高级建模:");
                
                // 这里添加了 NURBS 球体生成逻辑
                if ui.button("NURBS 球体") {
                    let r = 1.0; // 半径
                    let w_corner = 0.70710678; // sqrt(2)/2

                    // V 方向参数 (5行: 南极 -> 赤道 -> 北极)
                    // 元组格式: (半径缩放系数, Y轴高度, V方向权重)
                    // 这是一个二次B样条构成的半圆
                    let v_params = vec![
                        (0.0, -r, 1.0),      // 南极 (半径0)
                        (r, -r, w_corner),   // 南方角落控制点 (y=-r, radius=r)
                        (r, 0.0, 1.0),       // 赤道 (y=0, radius=r)
                        (r, r, w_corner),    // 北方角落控制点 (y=r, radius=r)
                        (0.0, r, 1.0),       // 北极 (半径0)
                    ];

                    // U 方向参数 (9列: 0度 -> 360度)
                    // 元组格式: (X方向系数, Z方向系数, U方向权重)
                    // 这是一个二次B样条构成的整圆 (正方形控制网格)
                    let u_params = vec![
                        (1.0, 0.0, 1.0),       // 0度
                        (1.0, 1.0, w_corner),  // 45度
                        (0.0, 1.0, 1.0),       // 90度
                        (-1.0, 1.0, w_corner), // 135度
                        (-1.0, 0.0, 1.0),      // 180度
                        (-1.0, -1.0, w_corner),// 225度
                        (0.0, -1.0, 1.0),      // 270度
                        (1.0, -1.0, w_corner), // 315度
                        (1.0, 0.0, 1.0),       // 360度(回到起点)
                    ];

                    let mut control_points = Vec::new();
                    let mut weights = Vec::new();

                    // 生成 9x5 = 45 个控制点网格
                    for v_p in &v_params {
                        for u_p in &u_params {
                            // 坐标计算: P = (x*rad, y, z*rad)
                            let x = u_p.0 * v_p.0; 
                            let z = u_p.1 * v_p.0;
                            let y = v_p.1;
                            
                            control_points.push([x, y, z]);
                            
                            // 权重计算: W = W_u * W_v
                            weights.push(u_p.2 * v_p.2);
                        }
                    }

                    // 节点向量 (2阶)
                    // U: 4段圆弧 (0, 0.25, 0.5, 0.75, 1) -> 需要重复节点来实现圆的完美拼接
                    let u_knots = vec![
                        0.0, 0.0, 0.0, 
                        0.25, 0.25, 
                        0.5, 0.5, 
                        0.75, 0.75, 
                        1.0, 1.0, 1.0
                    ];
                    // V: 2段圆弧 (0, 0.5, 1) -> 构成半圆
                    let v_knots = vec![
                        0.0, 0.0, 0.0, 
                        0.5, 0.5, 
                        1.0, 1.0, 1.0
                    ];

                    let mut obj = GameObject::new(
                        "NURBS Sphere",
                        Box::new(NurbsSurface {
                            degree: 2,
                            control_points,
                            weights,
                            u_count: 9,
                            v_count: 5,
                            splits: 32, // 细分度，越高越圆滑
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
                    // 在原点偏左一点生成
                    self.create_door(glam::vec3(-2.0, 1.0, 0.0));
                }
                ui.same_line();
                if ui.button("生成窗 (按F击碎)") {
                    // 在原点偏右一点生成
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

        // 先用只读方式获取位置，避免借用冲突
        let mut interact_pos = None;
        if let Some(idx) = self.get_selected_camera() {
            let pos = self.cameras[idx].camera.transform.position;
            // 检测按键，但不立即调用 handle_interaction_input
            if ui.is_key_pressed(imgui::Key::E) || ui.is_key_pressed(imgui::Key::F) {
                interact_pos = Some(pos);
            }
        }

        // 现在 safe 地调用交互逻辑 
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
                    let image: glium::texture::RawImage2d<'_, u8> =
                        display.read_front_buffer().unwrap();
                    let image = image::ImageBuffer::from_raw(
                        image.width,
                        image.height,
                        image.data.into_owned(),
                    )
                    .unwrap();
                    let image = image::DynamicImage::ImageRgba8(image).flipv();
                    image
                        .save(
                            "/saved/screenshot/screenshot_".to_owned()
                                + &timestamp.to_string()
                                + ".png",
                        )
                        .unwrap();
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

        // 顶点选择
        // if let (Some(mouse_click_near), Some(mouse_click_far)) = (mouse_click_near, mouse_click_far)
        // {
        //     if let Some(game_obj) = self.get_selected_mut() {
        //         if game_obj.kind == ShapeKind::Imported {
        //             let inv_model = game_obj.transform.get_matrix().inverse();
        //             let local_mouse_click_near = inv_model * mouse_click_near.extend(1.0);
        //             let local_mouse_click_far = inv_model * mouse_click_far.extend(1.0);
        //             let local_ray_o = local_mouse_click_near.truncate();
        //             let local_ray_d = (local_mouse_click_far.truncate() - local_ray_o).normalize();
        //             let camera_ray = ray::Ray {
        //                 o: local_ray_o,
        //                 d: local_ray_d,
        //                 t_max: f32::INFINITY,
        //             };
        //             match game_obj
        //                 .mesh
        //                 .compute_closest_point(camera_ray.o.to_array(), camera_ray.d.to_array())
        //             {
        //                 Some((pt, cos_angle)) => {
        //                     // 找到最近点，标记选中
        //                     if cos_angle < 0.99 {
        //                         game_obj.selected_vertex_index = None;
        //                         return;
        //                     }
        //                     for (i, v) in game_obj.mesh.vertices.iter().enumerate() {
        //                         let v_pos = glam::Vec3::from(*v);
        //                         if (v_pos - glam::Vec3::from(pt)).length() < 0.1 {
        //                             game_obj.selected_vertex_index = Some(i);
        //                             println!("selected point: {:?}", pt);
        //                             break;
        //                         }
        //                     }
        //                 }
        //                 None => {
        //                     game_obj.selected_vertex_index = None;
        //                     println!("no point found");
        //                 }
        //             }
        //         }
        //     }
        // }
    }
}
