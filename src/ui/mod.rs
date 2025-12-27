pub mod ctx;
pub mod ui_impl;
use glutin::surface::WindowSurface;
pub trait UIBuild {
    fn build_ui(&mut self, ui: &imgui::Ui);
}

pub trait UIHandle {
    fn handle_ui_input(&mut self, ui: &mut imgui::Ui, display: &glium::Display<WindowSurface>);
}
