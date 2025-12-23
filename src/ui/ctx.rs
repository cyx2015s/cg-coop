use imgui::{ FontConfig, FontGlyphRanges, FontSource};
use std::time::Instant;
use glutin::surface::WindowSurface;

pub struct GlobalContext {
    pub ui_ctx: imgui::Context,
    pub ui_renderer: imgui_glium_renderer::Renderer,
    pub ui_platform: imgui_winit_support::WinitPlatform,
    pub ui_last_frame_time: std::time::Instant,
    pub cn_font: imgui::FontId,
}

impl GlobalContext {
    pub fn new(display :&glium::Display<WindowSurface>, window:& glium::winit::window::Window) -> Self {
        let mut ui_ctx = imgui::Context::create();
        let mut ui_renderer = imgui_glium_renderer::Renderer::new(&mut ui_ctx, display).unwrap();
        let mut ui_platform = imgui_winit_support::WinitPlatform::new(&mut ui_ctx);
        let ui_last_frame_time = std::time::Instant::now();
        let cn_font = ui_ctx.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../../assets/fonts/font.ttf"),
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

        Self {
            ui_ctx,
            ui_renderer,
            ui_platform,
            ui_last_frame_time,
            cn_font,
        }
    }

    pub fn handle_event(&mut self, event: &glium::winit::event::Event<()>, window: &glium::winit::window::Window) {
        self.ui_platform.handle_event(self.ui_ctx.io_mut(), window, event);
    }

    pub fn update_time(&mut self) {
        self.ui_ctx.io_mut().update_delta_time(Instant::now() - self.ui_last_frame_time);
        self.ui_last_frame_time = Instant::now();
    }
}