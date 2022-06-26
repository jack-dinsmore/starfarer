mod menus;

use std::path::Path;
use std::rc::Rc;

use lepton::prelude::*;
use cgmath::{prelude::*, Vector3, Quaternion, Point3};

const WINDOW_TITLE: &'static str = "Starfarer";
const MODEL_PATH: &'static str = "assets/endeavour/accessories/port.obj";//"assets/chalet.obj";
const TEXTURE_PATH: &'static str = "assets/endeavour/accessories/port.png";//"assets/chalet.jpg";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const SENSITIVITY: f32 = 0.003;

struct Starfarer {
    model_shader: Shader<builtin::ModelSignature>,
    ui_shader: Shader<builtin::UISignature>,
    pattern: Pattern,

    camera: Camera,
    lights: Lights,
    key_tracker: KeyTracker,
    physics: Physics,

    docking_port: Object,
    docking_port2: Object,
    sun: Object,

    fps_menu: UserInterface<menus::FPS>,
    escape_menu: UserInterface<menus::Escape>,
    set_cursor_visible: bool,
}

impl Starfarer {
    fn new(graphics: &mut Graphics) -> Self {
        let pattern = Pattern::new(graphics);
        let model_shader = Shader::new(graphics);
        let ui_shader = Shader::new(graphics);
        let camera = Camera::new(graphics, Point3::new(2.0, 0.0, 1.0));
        let mut lights = Lights::new(graphics);
        let menu_common = menus::Common::new(graphics, &ui_shader);
        let fps_menu = menus::FPS::new(&menu_common);
        let escape_menu = menus::Escape::new(&menu_common);
        
        let physics = Physics::new();

        let ship_model = Rc::new(Model::new(graphics, &model_shader,
            VertexType::<vertex::VertexModel>::Path(&Path::new(MODEL_PATH)), TextureType::Transparency(&Path::new(TEXTURE_PATH)))
            .expect("Model creation failed"));

        let mut docking_port = Object::new(Vector3::new(0.0, 0.0, 0.0), Quaternion::new(1.0, 0.0, 0.0, 0.0));
        let mut docking_port2 = Object::new(Vector3::new(0.0, 0.0, 0.0), Quaternion::new(1.0, 0.0, 0.0, 0.0));
        docking_port.add_model(ship_model.clone());
        docking_port2.add_model(ship_model.clone());

        let mut sun = Object::new(Vector3::new(5.0, -5.0, 10.0), Quaternion::new(1.0, 0.0, 0.0, 0.0));
        lights.illuminate(&mut sun, LightFeatures { diffuse_coeff: 1.0, specular_coeff: 1.0, shininess: 1});

        Self {
            model_shader,
            ui_shader,
            pattern,
            camera,
            lights,
            fps_menu,
            escape_menu,
            key_tracker: KeyTracker::new(),
            physics,
            docking_port,
            docking_port2,
            sun,
            set_cursor_visible: false,
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
        self.docking_port2.set_pos(self.docking_port2.pos - Vector3::unit_y() * delta_time as f64);

        self.fps_menu.data.update(delta_time, &mut self.fps_menu.elements);
    }
    
    fn render(&mut self, graphics: &Graphics, render_data: &mut RenderData) {
        if self.set_cursor_visible {
            graphics.set_cursor_visible(self.escape_menu.data.is_open);
        }
        // Update inputs
        self.docking_port.update_light(&mut self.lights, None);
        self.camera.update_input(render_data.buffer_index);
        self.lights.update_input(render_data.buffer_index);

        let mut actions = vec![
            Action::LoadShader(&self.model_shader),
            Action::DrawObject(&mut self.docking_port),
            Action::DrawObject(&mut self.docking_port2),
            Action::LoadShader(&mut self.ui_shader),
            Action::DrawUI(&self.fps_menu),
        ];

        if self.escape_menu.data.is_open {
            actions.push(Action::DrawUI(&self.escape_menu));
        }

        // Record
        self.pattern.record(graphics, render_data.buffer_index, &mut actions);

        // Actually render
        self.pattern.render(render_data);
    }
    
    fn key_down(&mut self, vk: VirtualKeyCode) {
        self.key_tracker.key_down(vk);
        if let VirtualKeyCode::Escape = vk {
            if self.escape_menu.data.is_open {
                self.escape_menu.data.quit = true;
            }
            self.escape_menu.data.is_open = true;
            self.set_cursor_visible = true;
        }
    }
    
    fn key_up(&mut self, vk: VirtualKeyCode) {
        self.key_tracker.key_up(vk);
    }

    fn mouse_motion(&mut self, delta: (f64, f64)) -> bool {
        if !self.escape_menu.data.is_open {
            self.camera.turn(-delta.1 as f32 * SENSITIVITY, -delta.0 as f32 * SENSITIVITY);
            true
        } else {
            false
        }
    }

    fn mouse_down(&mut self, position: (f32, f32), _button: MouseButton) {
        if self.escape_menu.data.is_open {
            self.escape_menu.mouse_down(position)
        }
    }

    fn should_quit(&self) -> bool {
        self.escape_menu.data.quit
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
        vec![InputType::Camera, InputType::Lights, InputType::UI], 3);
    
    let starfarer = Starfarer::new(&mut graphics);
    
    control.run(graphics, starfarer);
}