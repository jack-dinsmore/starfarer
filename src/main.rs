use std::path::Path;

use lepton::{Graphics, Control, Renderer, KeyEvent, Pattern};
use lepton::{ElementState, VirtualKeyCode};
use lepton::model::Model;

const MODEL_PATH: &'static str = "assets/chalet.obj";
const TEXTURE_PATH: &'static str = "assets/chalet.jpg";


struct MyRenderer {
    pattern: Pattern,
}

impl MyRenderer {
    fn new(pattern: Pattern) -> MyRenderer {
        MyRenderer {pattern}
    }
}

impl Renderer for MyRenderer {
    fn get_pattern(&self) -> &Pattern {
        &self.pattern
    }
}

impl Drop for MyRenderer {
    fn drop(&mut self) {
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

fn main() {
    let control = Control::new();
    let graphics = Graphics::new(&control);
    let key_event = MyKeyEvent::new();
    
    let model = Model::new(&graphics, &Path::new(MODEL_PATH), &Path::new(TEXTURE_PATH)).expect("Model creation failed");
    
    let pattern = Pattern::begin(&graphics);
    pattern.render(&graphics, &model);
    let pattern = pattern.end(&graphics);

    let renderer = MyRenderer::new(pattern);

    control.run(graphics, Some(key_event), renderer, true);
}
// -------------------------------------------------------------------------------------------
