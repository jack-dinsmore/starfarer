use std::path::Path;

use lepton::prelude::*;
use cgmath::{prelude::*, Vector3, Quaternion, Matrix4};

const WINDOW_TITLE: &'static str = "Starfarer";
const MODEL_PATH: &'static str = "assets/endeavour/accessories/port.obj";//"assets/chalet.obj";
const TEXTURE_PATH: &'static str = "assets/endeavour/accessories/port.png";//"assets/chalet.jpg";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const SENSITIVITY: f32 = 0.003;

struct Starfarer {
    camera: Camera,
    lights: Lights,
    ui: UserInterface,
    key_tracker: KeyTracker,
    physics: Physics,
    data: StarfarerData,
    docking_port: Object,
    sun: Object,
    pos: Vector3<f32>,
}

impl Starfarer {
    fn new(graphics: &mut Graphics) -> Self {
        let ship_model = Model::new(graphics, &pattern, VertexType::Path(&Path::new(MODEL_PATH)), TextureType::Path(&Path::new(TEXTURE_PATH)))
            .expect("Model creation failed");

        let pattern = pattern.new(graphics);
        
        let pattern = pattern.end(graphics);
        pattern.record_fn = Some(Box::new(|pat: &mut Pattern<builtin::TextureShader>| {
            pat.models.push(ship_model);
        }));

        let camera = Camera::new(graphics);
        let physics = Physics::new();
        let mut lights = Lights::new();
        let docking_port = Object::new(
            Vector3::new(0.0, 0.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        );
        let mut sun = Object::new(
            Vector3::new(5.0, -5.0, 10.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
        );
        lights.illuminate(&mut sun, shader::LightFeatures {
            diffuse_coeff: 1.0,
            specular_coeff: 1.0,
            shininess: 1
        });
        let ui = UserInterface::new(graphics);

        Self {
            pattern,
            camera,
            lights,
            ui,
            key_tracker: KeyTracker::new(),
            physics,
            docking_port,
            sun,
            pos: Vector3::new(0.0, -1.0, 0.0),
        }
    }
}

impl Lepton for Starfarer {
    fn update(&mut self, delta_time: f32) {
        let mut camera_adjust = 
            - Vector3::unit_x() * ((self.key_tracker.get_state(VirtualKeyCode::W) as u32) as f32)
            + Vector3::unit_x() * ((self.key_tracker.get_state(VirtualKeyCode::S) as u32) as f32)
            - Vector3::unit_y() * ((self.key_tracker.get_state(VirtualKeyCode::A) as u32) as f32)
            + Vector3::unit_y() * ((self.key_tracker.get_state(VirtualKeyCode::D) as u32) as f32);
        if camera_adjust.magnitude() > 0.0 {
            camera_adjust *= delta_time / camera_adjust.magnitude();
        }
        self.camera.adjust(camera_adjust);
        self.pos.y += 0.01;

        // User interface
        unsafe {
            lepton::model::TEST_PUSH_CONSTANTS.model = Matrix4::from_translation(self.pos);
        }
    }
    
    fn render(&mut self, render_data: &mut RenderData) {
        // Update inputs
        self.docking_port.update_light(&mut self.lights, None);
        self.camera.update_input(render_data.buffer_index);
        self.lights.update_input(render_data.buffer_index);
        self.docking_port.update_input();

        // Record
        self.pattern.record(&[

        ])

        // Record
        self.pattern.record(&[

        ])

        // Actually render
        self.pattern.render(render_data);

        //self.ui.render(render_data);
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

    fn mouse_motion(&mut self, delta: (f64, f64)) -> bool {
        self.camera.turn(-delta.1 as f32 * SENSITIVITY, -delta.0 as f32 * SENSITIVITY);
        true
    }

    fn check_reload(&mut self, graphics: &Graphics) {
        //// Ideally, this would be moved inside a pattern call.
        self.pattern.check_reload(graphics);
        self.ui.check_reload(graphics);
    }
}

impl Drop for Starfarer {
    fn drop(&mut self) {
        println!("Starfarer dropped");
    }
}

fn main() {
    let control = Control::new();
    let mut graphics = Graphics::new(&control, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, true,
        vec![shader::InputType::Camera, shader::InputType::Lights], 2);
    
    let starfarer = Starfarer::new(&mut graphics);
    
    control.run(graphics, starfarer, true);
}