use glium::winit::keyboard::{KeyCode, PhysicalKey};
use std::collections::HashSet;

#[derive(Debug)]
pub struct InputState {
    pressed_keys: HashSet<PhysicalKey>,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
        }
    }

    pub fn is_key_pressed(&self, key: PhysicalKey) -> bool {
        self.pressed_keys.contains(&key)
    }

    // 为了方便，也提供 KeyCode 版本的检查
    pub fn is_keycode_pressed(&self, key_code: KeyCode) -> bool {
        self.pressed_keys.contains(&PhysicalKey::Code(key_code))
    }

    pub fn set_key_pressed(&mut self, key: PhysicalKey) {
        self.pressed_keys.insert(key);
    }

    pub fn set_key_released(&mut self, key: PhysicalKey) {
        self.pressed_keys.remove(&key);
    }

    pub fn clear(&mut self) {
        self.pressed_keys.clear();
    }

    // 获取所有按下的按键
    pub fn get_pressed_keys(&self) -> &HashSet<PhysicalKey> {
        &self.pressed_keys
    }

    // 只获取按下的 KeyCode（过滤掉其他类型的 PhysicalKey）
    pub fn get_pressed_keycodes(&self) -> HashSet<KeyCode> {
        self.pressed_keys
            .iter()
            .filter_map(|key| {
                if let PhysicalKey::Code(code) = key {
                    Some(*code)
                } else {
                    None
                }
            })
            .collect()
    }
}
