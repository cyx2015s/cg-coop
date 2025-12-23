mod scene;

use cg_coop::base::keystate::InputState;
use cg_coop::base::light::{
    AmbientLight, DirectionalLight, Light, LightBlock, PointLight, SpotLight,
};
use cg_coop::base::material;
use cg_coop::base::mouse;
use cg_coop::camera;
use cg_coop::shader;
use cg_coop::shape::mesh::{AsMesh, Mesh};
use glium::texture::{DepthFormat, DepthTexture2d, MipmapsOption};
use glium::winit::event::{DeviceEvent, Event, WindowEvent};
use glium::*;
use imgui::{ColorEdit3, Condition, Drag, FontConfig, FontGlyphRanges, FontSource, Slider};
use scene::{GameObject, Scene, ShapeKind};
use std::path::Path;
use std::time::Instant;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    texCoord: [f32; 2],
}
implement_vertex!(Vertex, position, texCoord);

#[derive(Copy, Clone)]
struct Normal {
    normal: [f32; 3],
}
implement_vertex!(Normal, normal);

fn main() {
    let phong_vertex_path = "assets/shaders/Phong.vert";
    let phong_fragment_path = "assets/shaders/Phong.frag";

    let default_mat =
        material::Phong::new([1.0, 0.5, 0.31], [1.0, 0.5, 0.31], [0.5, 0.5, 0.5], 32.0)
            .to_Material();

    let mut ambient_light = AmbientLight::new(0.2);
    let mut directional_light =
        DirectionalLight::new([0.0, 0.0, 1.0], [0.0, -2.0, -1.0], 1.5, [1.0, 1.0, 1.0]);
    let mut point_light = PointLight {
        position: [2.0, 2.0, 2.0],
        intensity: 0.0,
        color: [1.0, 1.0, 1.0],
        kc: 1.0,
        kl: 0.09,
        kq: 0.032,
    };
    let mut spot_light = SpotLight {
        position: [0.0, 5.0, 0.0],
        direction: [0.0, -1.0, 0.0],
        intensity: 10.0,
        color: [1.0, 1.0, 1.0],
        angle: 30.0,
        kc: 1.0,
        kl: 0.09,
        kq: 10.2,
    };

    let mut lambertian = material::Lambertian::new([1.0, 0.1, 0.1], [1.0, 0.1, 0.1]);

    let mut input_state = InputState::new();
    let mut last_frame_time = Instant::now();
    let global_ctx = cg_coop::ctx::GlobalContext {
        ui_ctx: imgui::Context::create(),
    };

    let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Project - Mini Blender Mode")
        .build(&event_loop);

    let light_block = LightBlock {
        lights: [Light::default(); 32],
        num_lights: 0,
    };
    let material_block = material::MaterialBlock {
        material: material::Material::default(),
    };
    let light_ubo = glium::uniforms::UniformBuffer::new(&display, light_block).unwrap();
    let material_ubo = glium::uniforms::UniformBuffer::new(&display, material_block).unwrap();

    let default_texture = {
        let size = 64; // 纹理总大小
        let check_size = 8; // 每个格子的大小 (8x8像素)
        let mut data = Vec::with_capacity(size * size * 4);

        for y in 0..size {
            for x in 0..size {
                // 根据坐标计算当前像素应该是黑还是白
                let is_white = ((x / check_size) + (y / check_size)) % 2 == 0;
                let color = if is_white { 255u8 } else { 0u8 };

                data.push(color);
                data.push(color);
                data.push(color);
                data.push(255);
            }
        }

        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&data, (size as u32, size as u32));
        glium::texture::SrgbTexture2d::new(&display, image).unwrap()
    };

    let loaded_texture: Option<glium::texture::SrgbTexture2d> = {
        let path_jpg = "assets/texture.jpg";
        let path_png = "assets/texture.png";

        let load_path = if Path::new(path_jpg).exists() {
            Some(path_jpg)
        } else if Path::new(path_png).exists() {
            Some(path_png)
        } else {
            None
        };

        if let Some(p) = load_path {
            println!("正在加载纹理: {}", p);
            match image::open(p) {
                Ok(img) => {
                    let img = img.flipv();
                    let img = img.to_rgba8();
                    let dims = img.dimensions();
                    let raw =
                        glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), dims);
                    Some(glium::texture::SrgbTexture2d::new(&display, raw).unwrap())
                }
                Err(e) => {
                    println!("纹理加载失败: {}", e);
                    None
                }
            }
        } else {
            None
        }
    };

    let mut ui_ctx = global_ctx.ui_ctx;
    let mut ui_renderer = imgui_glium_renderer::Renderer::new(&mut ui_ctx, &display).unwrap();
    let mut ui_platform = imgui_winit_support::WinitPlatform::new(&mut ui_ctx);
    let mut ui_last_frame_time = Instant::now();

    let cn_font = ui_ctx.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../assets/fonts/font.ttf"),
        size_pixels: 16.0 * window.scale_factor() as f32,
        config: Some(FontConfig {
            glyph_ranges: FontGlyphRanges::chinese_full(),
            ..Default::default()
        }),
    }]);
    ui_renderer
        .reload_font_texture(&mut ui_ctx)
        .expect("字体加载失败");
    ui_platform.attach_window(
        ui_ctx.io_mut(),
        &window,
        imgui_winit_support::HiDpiMode::Locked(1.0),
    );

    let (width, height) = display.get_framebuffer_dimensions();
    let mut camera = camera::Camera::new(width as f32 / height as f32);
    camera.transform.position = [0.0, 4.0, 10.0].into();
    camera
        .transform
        .look_at([0.0, 0.0, 0.0].into(), [0.0, 1.0, 0.0].into());
    camera.rotate(0.0, 0.0);
    let mut mouse_state = mouse::MouseState::new();

    let phong_program = shader::create_shader(&display, phong_vertex_path, phong_fragment_path);

    let mut scene = Scene::new();
    let mut selected_vertex_index: Option<usize> = None;

    let debug_sphere_mesh = cg_coop::shape::sphere::Sphere {
        radius: 0.05,
        col_divisions: 8,
        row_divisions: 8,
    }
    .as_mesh();

    let debug_sphere_verts: Vec<Vertex> = debug_sphere_mesh
        .vertices
        .iter()
        .zip(debug_sphere_mesh.tex_coords.iter())
        .map(|(v, t)| Vertex {
            position: *v,
            texCoord: *t,
        })
        .collect();
    let debug_sphere_norms: Vec<Normal> = debug_sphere_mesh
        .normals
        .iter()
        .map(|n| Normal { normal: *n })
        .collect();
    let debug_sphere_vbo = glium::VertexBuffer::new(&display, &debug_sphere_verts).unwrap();
    let debug_sphere_nbo = glium::VertexBuffer::new(&display, &debug_sphere_norms).unwrap();
    let debug_sphere_ibo = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &debug_sphere_mesh.indices,
    )
    .unwrap();

    let mut current_nurbs_idx: i32 = 0;

    let mut floor = GameObject::new(
        "Floor",
        ShapeKind::Cube {
            width: 10.0,
            height: 0.1,
            depth: 10.0,
        },
        default_mat,
    );
    floor.transform.position.y = -1.0;
    scene.add_object(floor);

    let sphere = GameObject::new(
        "Sphere",
        ShapeKind::Sphere {
            radius: 0.8,
            sectors: 32,
        },
        default_mat,
    );
    scene.add_object(sphere);

    let shadow_program = shader::create_shader(
        &display,
        "assets/shaders/Shadow.vert",
        "assets/shaders/Shadow.frag",
    );

    // 1. 创建阴影贴图
    let shadow_map_size = 2048;
    let shadow_texture = DepthTexture2d::empty_with_format(
        &display,
        DepthFormat::I24,
        MipmapsOption::NoMipmap,
        shadow_map_size,
        shadow_map_size,
    )
    .unwrap();

    #[allow(deprecated)]
    event_loop.run(move |ev, window_target| {
        ui_platform.handle_event(ui_ctx.io_mut(), &window, &ev);
        match ev {
            Event::DeviceEvent { device_id, event } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
                         if mouse_state.is_locked {
                            let (dx, dy) = delta;
                            mouse_state.delta = (dx as f32, dy as f32);
                            camera.rotate(-mouse_state.delta.0 * mouse_state.sensitivity, -mouse_state.delta.1 * mouse_state.sensitivity);
                            window.request_redraw();
                        }
                    },
                    _ => {}
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => {
                    let mut target = display.draw(); 

                    ui_ctx.io_mut().update_delta_time(Instant::now() - ui_last_frame_time);
                    ui_last_frame_time = Instant::now();
                    
                    let ui = ui_ctx.frame();
                    let _cn_font = ui.push_font(cn_font);

                    {
                        let scroll = ui.io().mouse_wheel;
                        camera.fovy *= f32::exp(-scroll * 0.005);
                        camera.fovy = camera.fovy.clamp(0.05, 1.5);
                    }

                    {
                        let current_time = Instant::now();
                        let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
                        last_frame_time = current_time;
                        let move_speed = 5.0 * delta_time;
                        
                        if camera.move_state == camera::MoveState::Free {
                            if ui.is_key_down(imgui::Key::W) { camera.transform.position += camera.transform.get_forward() * move_speed; }
                            if ui.is_key_down(imgui::Key::S) { camera.transform.position -= camera.transform.get_forward() * move_speed; }
                            if ui.is_key_down(imgui::Key::A) { camera.transform.position -= camera.transform.get_right() * move_speed; }
                            if ui.is_key_down(imgui::Key::D) { camera.transform.position += camera.transform.get_right() * move_speed; }
                            if ui.is_key_down(imgui::Key::Space) { camera.transform.position += glam::f32::Vec3::Y * move_speed; }
                            if ui.is_key_down(imgui::Key::LeftCtrl) { camera.transform.position -= glam::f32::Vec3::Y * move_speed; }
                        }
                    }

                    {
                        if ui.is_key_pressed(imgui::Key::V) {
                            camera.move_state = if camera.move_state == camera::MoveState::Free { camera::MoveState::Locked } else { camera::MoveState::Free };
                            mouse_state.toggle_lock(&window);                        
                        }
                        if ui.is_key_pressed(imgui::Key::B) {
                            if camera.move_state == camera::MoveState::PanObit {
                                camera.stop_pan_obit();
                            } else {
                                camera.start_pan_obit(0.0, 5.0, [0.5, 0.5, 0.5]);
                                camera.pan_obit_speed = 10.0;
                            }
                        }
                        if ui.is_key_pressed(imgui::Key::R) {
                            camera.fovy = 3.141592 / 2.0;
                        }
                        if ui.is_key_pressed(imgui::Key::P) {
                            let image: glium::texture::RawImage2d<'_, u8> = display.read_front_buffer().unwrap();
                            let image = image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
                            let image = image::DynamicImage::ImageRgba8(image).flipv();
                            image.save("screenshot.png").unwrap();
                        }
                    }

                    if camera.move_state == camera::MoveState::PanObit {
                        let current_time = Instant::now();
                        let delta_time = current_time.duration_since(last_frame_time).as_secs_f32();
                        camera.update_pan_obit(delta_time);
                    }


                    ui.show_demo_window(&mut true); 

                    ui.window("操作说明")
                        .size([300.0, 100.0], Condition::FirstUseEver)
                        .build(|| {
                            ui.text_wrapped("按V键漫游\n按B键环绕\n滚轮放大缩小视角\n按R恢复视角\n按P键截图\n按WS在摄像头方向前后移动\n按AD左右移动\n按Space/Ctrl上升下降");     
                        });

                    ui.window("灯光与材质测试").size([400.0, 200.0], Condition::FirstUseEver).build(|| {
                        Slider::new(ui, "环境光", 0.0, 1.0).build(&mut ambient_light.intensity);
                        Slider::new(ui, "平行光", 0.0, 5.0).build(&mut directional_light.intensity);
                        Slider::new(ui, "点光源", 0.0, 50.0).build(&mut point_light.intensity);
                        Slider::new(ui, "聚光灯", 0.0, 100.0).build(&mut spot_light.intensity);
                        
                        ui.separator();
                        Slider::new(ui, "平行光方向", -3.0, 3.0).build_array(&mut directional_light.direction);
                        ColorEdit3::new(ui, "平行光颜色", &mut directional_light.color).build();
                        
                        ui.separator();
                        Slider::new(ui, "点光源位置", -10.0, 10.0).build_array(&mut point_light.position);
                        ColorEdit3::new(ui, "点光源颜色", &mut point_light.color).build();
                        
                        ui.separator();
                        ui.text("全局材质预览");
                        ColorEdit3::new(ui, "材质 ka", &mut lambertian.ka).build();
                        ColorEdit3::new(ui, "材质 kd", &mut lambertian.kd).build();
                    });

                    ui.window("场景列表 (Scene List)").size([200.0, 400.0], Condition::FirstUseEver).position([20.0, 150.0], Condition::FirstUseEver).build(|| {
                        ui.text("基础形状:");
                        if ui.button("立方体") { scene.add_object(GameObject::new("Cube", ShapeKind::Cube{width:1.0, height:1.0, depth:1.0}, default_mat)); }
                        ui.same_line();
                        if ui.button("球体") { scene.add_object(GameObject::new("Sphere", ShapeKind::Sphere{radius:0.5, sectors:32}, default_mat)); }
                        
                        ui.text("柱体/台体:");
                        if ui.button("圆柱") { scene.add_object(GameObject::new("Cylinder", ShapeKind::Cylinder{top_radius:0.5, bottom_radius:0.5, height:1.0, sectors:32}, default_mat)); }
                        if ui.button("棱台") { scene.add_object(GameObject::new("Frustum", ShapeKind::Cylinder{top_radius:0.3, bottom_radius:0.8, height:1.0, sectors:32}, default_mat)); }
                        if ui.button("圆锥") { scene.add_object(GameObject::new("Cone", ShapeKind::Cone{radius:0.5, height:1.0, sectors:32}, default_mat)); }

                        ui.separator();
                        ui.text("高级建模:");
                        if ui.button("NURBS 曲面") {
                            let pts = vec![
                                [-1.5, 0.0, -1.5], [-0.5, 0.5, -1.5], [0.5, 0.5, -1.5], [1.5, 0.0, -1.5],
                                [-1.5, 0.5, -0.5], [-0.5, 1.5, -0.5], [0.5, 1.5, -0.5], [1.5, 0.5, -0.5],
                                [-1.5, 0.5,  0.5], [-0.5, 1.5,  0.5], [0.5, 1.5,  0.5], [1.5, 0.5,  0.5],
                                [-1.5, 0.0,  1.5], [-0.5, 0.5,  1.5], [0.5, 0.5,  1.5], [1.5, 0.0,  1.5],
                            ];
                            let weights = vec![1.0; 16];
                            scene.add_object(GameObject::new("Nurbs Surface", ShapeKind::Nurbs{
                                degree: 3, control_points: pts, weights, u_count: 4, v_count: 4
                            }, default_mat));
                        }
                        if ui.button("导入模型") {
                            if let Ok(mesh) = Mesh::load_obj("output.obj") {
                                let mut obj = GameObject::new("Imported", ShapeKind::Imported, default_mat);
                                obj.mesh = mesh;
                                scene.add_object(obj);
                            }
                        }
                        
                        ui.separator();
                        ui.text("场景物体:");
                        for (i, obj) in scene.objects.iter().enumerate() {
                            let is_selected = scene.selected_index == Some(i);
                            if ui.selectable_config(&format!("{}: {}", i, obj.name)).selected(is_selected).build() {
                                scene.selected_index = Some(i);
                            }
                        }
                    });

                    if let Some(obj) = scene.get_selected_mut() {
                        ui.window("属性面板 (Inspector)").size([250.0, 500.0], Condition::FirstUseEver).position([240.0, 150.0], Condition::FirstUseEver).build(|| {
                            ui.text_colored([0.0, 1.0, 0.0, 1.0], &format!("当前选中: {}", obj.name));
                            ui.separator();
                            ui.text("变换 (Transform)");
                            let mut pos = obj.transform.position.to_array();
                            if Drag::new("位置").speed(0.1).build_array(ui, &mut pos) { obj.transform.position = pos.into(); }
                            let mut scale = obj.transform.scale.to_array();
                            if Drag::new("缩放").speed(0.01).build_array(ui, &mut scale) { obj.transform.scale = scale.into(); }
                            if ui.button("重置旋转") { obj.transform.rotation = glam::f32::Quat::IDENTITY; }

                            ui.separator();
                            ui.text("形状参数 (Parameters)");
                            
                            let mut need_regen = false;
                            
                            match &mut obj.kind {
                                ShapeKind::Cube { width, height, depth } => {
                                    if Drag::new("宽").speed(0.1).build(ui, width) { need_regen = true; }
                                    if Drag::new("高").speed(0.1).build(ui, height) { need_regen = true; }
                                    if Drag::new("深").speed(0.1).build(ui, depth) { need_regen = true; }
                                },
                                ShapeKind::Sphere { radius, sectors } => {
                                    if Drag::new("半径").speed(0.05).build(ui, radius) { need_regen = true; }
                                    let mut s = *sectors as i32;
                                    if Slider::new(ui, "精度", 3, 64).build(&mut s) { *sectors = s as u16; need_regen = true; }
                                },
                                ShapeKind::Cylinder { top_radius, bottom_radius, height, sectors } => {
                                    if Drag::new("顶半径").speed(0.05).build(ui, top_radius) { need_regen = true; }
                                    if Drag::new("底半径").speed(0.05).build(ui, bottom_radius) { need_regen = true; }
                                    if Drag::new("高度").speed(0.1).build(ui, height) { need_regen = true; }
                                    let mut s = *sectors as i32;
                                    if Slider::new(ui, "精度", 3, 64).build(&mut s) { *sectors = s as u16; need_regen = true; }
                                },
                                ShapeKind::Cone { radius, height, sectors } => {
                                    if Drag::new("底半径").speed(0.05).build(ui, radius) { need_regen = true; }
                                    if Drag::new("高度").speed(0.1).build(ui, height) { need_regen = true; }
                                    let mut s = *sectors as i32;
                                    if Slider::new(ui, "精度", 3, 64).build(&mut s) { *sectors = s as u16; need_regen = true; }
                                },
                                ShapeKind::Nurbs { control_points, weights, .. } => {
                                    ui.text("NURBS 控制点编辑");
                                    Slider::new(ui, "点索引", 0, 15).build(&mut current_nurbs_idx);
                                    let idx = current_nurbs_idx as usize;
                                    if idx < control_points.len() {
                                        ui.text_colored([1.0, 1.0, 0.0, 1.0], "编辑中...");
                                        if Drag::new("X").speed(0.05).build(ui, &mut control_points[idx][0]) { need_regen = true; }
                                        if Drag::new("Y").speed(0.05).build(ui, &mut control_points[idx][1]) { need_regen = true; }
                                        if Drag::new("Z").speed(0.05).build(ui, &mut control_points[idx][2]) { need_regen = true; }
                                        if Drag::new("权重").speed(0.05).range(0.1, 100.0).build(ui, &mut weights[idx]) { need_regen = true; }
                                    }
                                },
                                _ => {}
                            }
                            
                            if need_regen {
                                obj.regenerate_mesh();
                            }

                            if obj.kind != ShapeKind::Imported {
                                if (ui.button("网格化（不可逆！）")) {
                                    *obj = GameObject {
                                        name: obj.name.clone() + " (Meshed)",
                                        transform: obj.transform.clone(),
                                        material: obj.material,
                                        mesh: obj.mesh.clone(),
                                        kind: ShapeKind::Imported,
                                        visible: obj.visible,
                                        use_texture: obj.use_texture,
                                    }
                                    
                                }
                            } else {
                                ui.text("模型已网格化，只能编辑顶点位置和UV映射。");
                                if ui.is_mouse_clicked(imgui::MouseButton::Left) && !ui.is_any_item_focused() && !ui.is_any_item_hovered(){
                                    // 尝试选中顶点
                                    let mouse_pos = ui.io().mouse_pos;
                                    let [win_w, win_h] = ui.io().display_size;
                                    let ndc_x = (2.0 * mouse_pos[0]) / win_w - 1.0;
                                    let ndc_y = 1.0 - (2.0 * mouse_pos[1]) / win_h;
                                    let ndc_pos = glam::Vec3::new(ndc_x, ndc_y, 1.0);
                                    let inv_proj = glam::Mat4::from_cols_array_2d(&camera.get_projection_matrix()).inverse();
                                    let inv_view = glam::Mat4::from_cols_array_2d(&camera.get_view_matrix()).inverse();
                                    let ray_dir_camera = (inv_proj * ndc_pos.extend(1.0)).truncate().normalize();
                                    let ray_dir_world = (inv_view * ray_dir_camera.extend(0.0)).truncate().normalize();
                                    let ray_origin = glam::Vec3::from(camera.get_position());
                                    match obj.mesh.compute_closest_point(
                                        ray_origin.to_array(),
                                        ray_dir_world.to_array()
                                    ) {
                                        Some((pt, cos_angle)) => {
                                            // 找到最近点，标记选中
                                            if cos_angle < 0.99 {
                                                selected_vertex_index = None;
                                                return;
                                            }
                                            for (i, v) in obj.mesh.vertices.iter().enumerate() {
                                                let v_pos = glam::Vec3::from(*v);
                                                if (v_pos - glam::Vec3::from(pt)).length() < 0.1 {
                                                    selected_vertex_index = Some(i);
                                                    break;
                                                }
                                            }
                                        },
                                        None => {
                                            selected_vertex_index = None;
                                        }
                                    }
                                }
                            }
                            match selected_vertex_index {
                                Some(idx) => {
                                    ui.text_colored([1.0, 1.0, 0.0, 1.0], &format!("编辑顶点 {}", idx));
                                    let v = &mut obj.mesh.vertices[idx];
                                    if Drag::new("X").speed(0.01).build(ui, &mut v[0]) {}
                                    if Drag::new("Y").speed(0.01).build(ui, &mut v[1]) {}
                                    if Drag::new("Z").speed(0.01).build(ui, &mut v[2]) {}
                                    
                                    let t = &mut obj.mesh.tex_coords[idx];
                                    if Drag::new("U").speed(0.01).build(ui, &mut t[0]) {}
                                    if Drag::new("V").speed(0.01).build(ui, &mut t[1]) {}
                                },
                                None => {
                                    ui.text("未选中顶点");
                                }
                            }
                            ui.separator();
                            ui.checkbox("显示/隐藏", &mut obj.visible);
                            ui.checkbox("启用纹理贴图", &mut obj.use_texture);
                            if ui.button("保存当前模型") { let _ = obj.mesh.save_obj("output.obj"); }
                        });
                    }

                    // 更新灯光 UBO
                    let mut l_block = LightBlock { lights: [Light::default(); 32], num_lights: 0 };
                    l_block.lights[0] = ambient_light.to_Light(); l_block.num_lights += 1;
                    l_block.lights[1] = directional_light.to_Light(); l_block.num_lights += 1;
                    l_block.lights[2] = point_light.to_Light(); l_block.num_lights += 1;
                    l_block.lights[3] = spot_light.to_Light(); l_block.num_lights += 1; 
                    light_ubo.write(&l_block);

                    // 生成 Shadow Map
                    
                    // 计算灯光空间矩阵
                    let light_dir = glam::Vec3::from(directional_light.direction).normalize();
                    let light_pos = glam::Vec3::ZERO - light_dir * 20.0; 
                    
                    let light_projection = glam::Mat4::orthographic_rh(-20.0, 20.0, -20.0, 20.0, 1.0, 50.0);
                    let light_view = glam::Mat4::look_at_rh(light_pos, glam::Vec3::ZERO, glam::Vec3::Y);
                    let light_space_matrix = light_projection * light_view;
                    let light_space_arr = light_space_matrix.to_cols_array_2d();

                    // 在作用域内绘制到 shadow texture
                    {
                        let mut shadow_target = glium::framebuffer::SimpleFrameBuffer::depth_only(&display, &shadow_texture).unwrap();
                        shadow_target.clear_depth(1.0);

                        let shadow_params = glium::DrawParameters {
                            depth: glium::Depth {
                                test: glium::draw_parameters::DepthTest::IfLess,
                                write: true,
                                .. Default::default()
                            },
                            .. Default::default()
                        };

                        for obj in &scene.objects {
                            if !obj.visible { continue; }
                            let model = obj.transform.get_matrix().to_cols_array_2d();
                            
                            let vertex_data: Vec<Vertex> = obj.mesh.vertices.iter()
                                .zip(obj.mesh.tex_coords.iter())
                                .map(|(v, t)| Vertex { position: *v, texCoord: *t })
                                .collect();
                            
                            if vertex_data.is_empty() { continue; }
                            let positions = glium::VertexBuffer::new(&display, &vertex_data).unwrap();
                            let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &obj.mesh.indices).unwrap();

                            shadow_target.draw(
                                &positions, 
                                &indices, 
                                &shadow_program,
                                &uniform! {
                                    model: model,
                                    light_space_matrix: light_space_arr,
                                },
                                &shadow_params
                            ).unwrap();
                        }
                    }

                    // 正常绘制场景
                    target.clear_color_and_depth((0.05, 0.05, 0.1, 1.0), 1.0); 

                    let view = camera.get_view_matrix();
                    let perspective = camera.get_projection_matrix();
                    let viewPos = camera.get_position();

                    let params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            .. Default::default()
                        },
                        .. Default::default()
                    };
                    
                    // 设置阴影贴图采样器
                    let shadow_sampler = glium::uniforms::Sampler::new(&shadow_texture)
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                        .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest)
                        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp);

                    for obj in &scene.objects {
                        if !obj.visible { continue; }
                        let model = obj.transform.get_matrix().to_cols_array_2d();
                        let m_block = material::MaterialBlock { material: obj.material };
                        material_ubo.write(&m_block);

                        let vertex_data: Vec<Vertex> = obj.mesh.vertices.iter()
                            .zip(obj.mesh.tex_coords.iter())
                            .map(|(v, t)| Vertex { position: *v, texCoord: *t })
                            .collect();
                        let normal_data: Vec<Normal> = obj.mesh.normals.iter().map(|n| Normal { normal: *n }).collect();
                        
                        if vertex_data.is_empty() { continue; }

                        let positions = glium::VertexBuffer::new(&display, &vertex_data).unwrap();
                        let normals = glium::VertexBuffer::new(&display, &normal_data).unwrap();
                        let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &obj.mesh.indices).unwrap();

                        let use_tex = if let Some(tex) = &loaded_texture {
                            tex
                        } else {
                            &default_texture
                        };

                        target.draw((&positions, &normals), &indices, &phong_program,
                            &uniform! { 
                                model: model, 
                                view: view, 
                                perspective: perspective,
                                viewPos: viewPos,
                                Material_Block: &material_ubo,
                                Light_Block: &light_ubo,
                                diffuse_tex: use_tex, 
                                has_texture: obj.use_texture,
                                // 传入阴影参数
                                light_space_matrix: light_space_arr,
                                shadow_map: shadow_sampler,
                            },
                            &params).unwrap();

                        // 绘制 NURBS Debug 小球
                        if let ShapeKind::Nurbs { control_points, .. } = &obj.kind {
                            if scene.selected_index == Some(scene.objects.iter().position(|x| std::ptr::eq(x, obj)).unwrap_or(999)) {
                                for (idx, pt) in control_points.iter().enumerate() {
                                    let pt_local = glam::f32::Vec3::from(*pt);
                                    let obj_matrix = obj.transform.get_matrix(); 
                                    let world_pos = obj_matrix.transform_point3(pt_local);
                                    let sphere_model = glam::f32::Mat4::from_translation(world_pos).to_cols_array_2d();
                                    
                                    let is_active = idx == current_nurbs_idx as usize;
                                    let debug_color = if is_active { [1.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
                                    let debug_mat = material::Phong::new(debug_color, debug_color, [0.0,0.0,0.0], 1.0).to_Material();
                                    let debug_m_block = material::MaterialBlock { material: debug_mat };
                                    material_ubo.write(&debug_m_block);

                                    target.draw((&debug_sphere_vbo, &debug_sphere_nbo), &debug_sphere_ibo, &phong_program,
                                        &uniform! { 
                                            model: sphere_model, 
                                            view: view, 
                                            perspective: perspective,
                                            viewPos: viewPos,
                                            Material_Block: &material_ubo,
                                            Light_Block: &light_ubo,
                                            diffuse_tex: use_tex, 
                                            has_texture: false,
                                            // 即使不需要阴影，也要传参以匹配 Shader 接口
                                            light_space_matrix: light_space_arr,
                                            shadow_map: shadow_sampler,
                                        },
                                        &params).unwrap();
                                }
                            }
                        }
                    }

                    _cn_font.pop();
                    let draw_data = ui_ctx.render();
                    if draw_data.draw_lists_count() > 0 {
                        ui_renderer.render(&mut target, draw_data).unwrap();
                    }
                    target.finish().unwrap();
                }
                winit::event::WindowEvent::CloseRequested => { window_target.exit(); }
                _ => (),
            },
            Event::AboutToWait => {
                window.request_redraw();
            },
            _ => (),
        }
    }).unwrap();
}
