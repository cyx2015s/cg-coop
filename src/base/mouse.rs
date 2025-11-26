#[derive(Debug)]
pub struct MouseState {
    pub position: (f64, f64),
    pub delta: (f32, f32),
    pub is_locked: bool,
    pub is_visible: bool,
    pub sensitivity: f32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            delta: (0.0, 0.0),
            is_locked: false,
            is_visible: true,
            sensitivity: 0.01,
        }
    }

    pub fn toggle_lock(&mut self, window: &glium::winit::window::Window) -> bool {
        self.is_locked = !self.is_locked;
        self.is_visible = !self.is_visible;

        if self.is_locked {
            // 尝试锁定
            if window
                .set_cursor_grab(glium::winit::window::CursorGrabMode::Confined)
                .is_ok()
                || window
                    .set_cursor_grab(glium::winit::window::CursorGrabMode::Locked)
                    .is_ok()
            {
                window.set_cursor_visible(false);

                println!("鼠标已锁定");
                true
            } else {
                self.is_locked = false;
                println!("无法锁定鼠标");
                false
            }
        } else {
            // 释放
            let _ = window.set_cursor_grab(glium::winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
            println!("鼠标已释放");
            true
        }
    }
}
