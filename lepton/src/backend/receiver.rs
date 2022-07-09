pub use winit::event::{VirtualKeyCode, MouseButton};

pub struct KeyTracker {
    low_mask: u128,
    high_mask: u128,
}

impl KeyTracker {
    pub fn new() -> KeyTracker {
        KeyTracker {
            low_mask: 0,
            high_mask: 0,
        }
    }

    pub fn key_down(&mut self, vk: winit::event::VirtualKeyCode) {
        if (vk as u32) < 128 {
            self.low_mask |= 1 << (vk as u32);
        } else {
            self.high_mask |= 1 << ((vk as u32) - 128);
        }
    }

    pub fn key_up(&mut self, vk: winit::event::VirtualKeyCode) {
        if (vk as u32) < 128 {
            self.low_mask &= !(1 << (vk as u32));
        } else {
            self.high_mask &= !(1 << ((vk as u32) - 128));
        }
    }

    pub fn get_state(&self, vk: winit::event::VirtualKeyCode) -> bool{
        0 != if (vk as u32) < 128 {
            self.low_mask & (1 << (vk as u32))
        } else {
            self.high_mask & (1 << ((vk as u32) - 128))
        }
    }
}

pub trait InputReceiver {
    /// Respond to a key press. Returns true if the program is to exit.
    fn key_down(&mut self, _keycode: VirtualKeyCode) {}

    fn mouse_down(&mut self, _position: (f32, f32), _button: MouseButton) {}

    /// Respond to a key release
    fn key_up(&mut self, _keycode: VirtualKeyCode) {}

    /// Respond to mouse motion. True if the mouse pointer is to be reset to the center.
    fn mouse_motion(&mut self, _delta: (f64, f64)) -> bool {false}
}