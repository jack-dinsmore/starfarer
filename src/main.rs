use lepton::{Graphics, Control, Renderer, KeyEvent};
use winit::event::{ElementState, VirtualKeyCode};

struct MyRenderer {
    num: &'static f32
}

impl MyRenderer {
    fn new(num: &'static f32) -> MyRenderer {
        MyRenderer {num}
    }
}

impl Renderer for MyRenderer {
    fn draw_frame(&mut self, delta_time: f32) {
        
    }
}

impl Drop for MyRenderer {
    fn drop(&mut self) {
        println!("Shared number {}", self.num);
        println!("Renderer dropped");
    }
}

struct MyKeyEvent {
}

impl MyKeyEvent {
    fn new() -> Self {
        MyKeyEvent {}
    }
}

impl KeyEvent for MyKeyEvent {
    fn keydown(&mut self, virtual_keycode: Option<VirtualKeyCode>, state: ElementState) -> bool {
        match (virtual_keycode, state) {
            | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                true
            },
            | _ => false,
        }
    }
}

static shared_number: f32 = 7.2;

fn main() {
    let control = Control::new();
    let graphics = Graphics::new(&control);
    let renderer = MyRenderer::new(&shared_number);
    let key_event = MyKeyEvent::new();

    control.run(graphics, Some(key_event), renderer, true);
}
// -------------------------------------------------------------------------------------------
