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
    shader: Shader,
    pattern: Pattern,
    camera: Camera,
    lights: Lights,
    ui: UserInterface,
    key_tracker: KeyTracker,
    physics: Physics,
    docking_port: Object,
    sun: Object,
}

impl Starfarer {
    fn new(graphics: &mut Graphics) -> Self {
        let pattern = Pattern::new(graphics);
        let shader = Shader::new::<builtin::TextureShader>(graphics);
        let camera = Camera::new(graphics);
        let mut lights = Lights::new();
        let ui = UserInterface::new(graphics);
        
        let physics = Physics::new();

        let ship_model = Model::new::<builtin::TextureShader>(graphics, &shader,
            VertexType::Path(&Path::new(MODEL_PATH)), TextureType::Path(&Path::new(TEXTURE_PATH)))
            .expect("Model creation failed");

        let mut docking_port = Object::new(Vector3::new(0.0, 0.0, 0.0), Quaternion::new(1.0, 0.0, 0.0, 0.0));
        docking_port.add_model(ship_model.clone());

        let mut sun = Object::new(Vector3::new(5.0, -5.0, 10.0), Quaternion::new(1.0, 0.0, 0.0, 0.0));
        lights.illuminate(&mut sun, shader::LightFeatures { diffuse_coeff: 1.0, specular_coeff: 1.0, shininess: 1});

        Self {
            pattern,
            shader,
            camera,
            lights,
            ui,
            key_tracker: KeyTracker::new(),
            physics,
            docking_port,
            sun,
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
        self.docking_port.set_pos(self.docking_port.pos + Vector3::unit_y() * delta_time as f64);

        // User interface
    }
    
    fn render(&mut self, graphics: &Graphics, render_data: &mut RenderData) {
        // Update inputs
        self.docking_port.update_light(&mut self.lights, None);
        self.camera.update_input(render_data.buffer_index);
        self.lights.update_input(render_data.buffer_index);

        // Record
        self.pattern.record(graphics, render_data.buffer_index, &mut vec![
            Action::LoadShader(&self.shader),
            Action::DrawObject(&mut self.docking_port),
        ]);

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