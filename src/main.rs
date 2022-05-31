use std::path::Path;

use lepton::{Graphics, Control, Lepton, Pattern, PatternTrait, shader::Camera};
use lepton::shader::CameraData;
use lepton::{VirtualKeyCode, KeyTracker};
use lepton::model::Model;
use cgmath::{prelude::*, Vector3};

const WINDOW_TITLE: &'static str = "Starfarer";
const MODEL_PATH: &'static str = "assets/chalet.obj";
const TEXTURE_PATH: &'static str = "assets/chalet.jpg";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;


struct Starfarer {
    pattern: Pattern<CameraData>,
    camera: Camera,
    key_tracker: KeyTracker,
}

impl Starfarer {
    fn new(graphics: &Graphics) -> Starfarer {

        let mut pattern = Pattern::begin(graphics);
        pattern.add(Model::new(graphics, &pattern, &Path::new(MODEL_PATH), &Path::new(TEXTURE_PATH))
            .expect("Model creation failed"));
        let pattern = pattern.end(graphics);

        let camera = Camera::new(graphics);

        Starfarer {
            pattern,
            camera,
            key_tracker: KeyTracker::new(),
        }
    }
}

impl Lepton for Starfarer {
    fn update(&mut self, delta_time: f32) {
        let mut camera_adjust = 
              Vector3::unit_x() * ((self.key_tracker.get_state(VirtualKeyCode::W) as u32) as f32)
            - Vector3::unit_x() * ((self.key_tracker.get_state(VirtualKeyCode::S) as u32) as f32)
            + Vector3::unit_y() * ((self.key_tracker.get_state(VirtualKeyCode::A) as u32) as f32)
            - Vector3::unit_y() * ((self.key_tracker.get_state(VirtualKeyCode::D) as u32) as f32);
        if camera_adjust.magnitude() > 0.0 {
            camera_adjust *= delta_time / camera_adjust.magnitude();
        }
        self.camera.adjust(camera_adjust);
        self.camera.update(self.pattern.uniform());
    }

    fn get_pattern(&mut self) -> &mut dyn PatternTrait {
        &mut self.pattern
    }
    
    fn keydown(&mut self, vk: VirtualKeyCode) -> bool {
        self.key_tracker.keydown(vk);
        if let VirtualKeyCode::Escape = vk {
            return true;
        }
        false
    }
    
    fn keyup(&mut self, vk: VirtualKeyCode) -> bool {
        self.key_tracker.keyup(vk);
        false
    }
}

impl Drop for Starfarer {
    fn drop(&mut self) {
        println!("Starfarer dropped");
    }
}

fn main() {
    let control = Control::new();
    let mut graphics = Graphics::new(&control, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
    let starfarer = Starfarer::new(&mut graphics);
    control.run(graphics, starfarer, true);
}