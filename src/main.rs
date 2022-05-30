use std::path::Path;

use lepton::{Graphics, Control, Lepton, Pattern, PatternTrait};
use lepton::shader::CameraData;
use lepton::{ElementState, VirtualKeyCode};
use lepton::model::Model;

const MODEL_PATH: &'static str = "assets/chalet.obj";
const TEXTURE_PATH: &'static str = "assets/chalet.jpg";


struct Starfarer {
    pattern: Pattern<CameraData>,
}

impl Starfarer {
    fn new(pattern: Pattern<CameraData>) -> Starfarer {
        Starfarer {pattern}
    }
}

impl Lepton for Starfarer {
    fn update(&mut self, delta_time: f32) {
        self.pattern.uniform().update(delta_time);
    }

    fn get_pattern(&self) -> &dyn PatternTrait {
        &self.pattern
    }
    
    fn keydown(&mut self, virtual_keycode: Option<VirtualKeyCode>, state: ElementState) -> bool {
        match (virtual_keycode, state) {
            | (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                true
            },
            | _ => false,
        }
    }
}

impl Drop for Starfarer {
    fn drop(&mut self) {
        println!("Starfarer dropped");
    }
}

fn main() {
    let control = Control::new();
    let graphics = Graphics::new(&control);
    
    let pattern = Pattern::begin(&graphics);
    Model::new(&graphics, &pattern, &Path::new(MODEL_PATH), &Path::new(TEXTURE_PATH)).expect("Model creation failed");
    let pattern = pattern.end(&graphics);

    let renderer = Starfarer::new(pattern);

    control.run(graphics, renderer, true);
}
// -------------------------------------------------------------------------------------------
