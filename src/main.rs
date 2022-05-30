use std::path::Path;

use lepton::{Graphics, Control, Renderer, KeyEvent, Pattern, PatternTrait};
use lepton::shader::{ShaderData, CameraData};
use lepton::{ElementState, VirtualKeyCode};
use lepton::model::Model;

const MODEL_PATH: &'static str = "assets/chalet.obj";
const TEXTURE_PATH: &'static str = "assets/chalet.jpg";


struct MyRenderer<D: ShaderData> {
    pattern: Pattern<D>,
}

impl<D: ShaderData> MyRenderer<D> {
    fn new(pattern: Pattern<D>) -> MyRenderer<D> {
        MyRenderer {pattern}
    }
}

impl<D: ShaderData> Renderer for MyRenderer<D> {
    fn update(&mut self, delta_time: f32) {
        self.pattern.update_uniform(delta_time); 
    }

    fn get_pattern(&self) -> &dyn PatternTrait {
        &self.pattern
    }
}

impl<D: ShaderData> Drop for MyRenderer<D> {
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
    
    let pattern = Pattern::begin(&graphics, CameraData::new(1.0));
    Model::new(&graphics, &pattern, &Path::new(MODEL_PATH), &Path::new(TEXTURE_PATH)).expect("Model creation failed");
    let pattern = pattern.end(&graphics);

    let renderer = MyRenderer::new(pattern);

    control.run(graphics, Some(key_event), renderer, true);
}
// -------------------------------------------------------------------------------------------
