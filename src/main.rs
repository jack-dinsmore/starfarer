#![allow(clippy::too_many_arguments)]

#[cfg(test)]
mod test;

mod menus;
mod ships;
mod skybox;
mod planet;
mod threadpool;

use skybox::Skybox;
use lepton::prelude::*;
use cgmath::{prelude::*, Vector3, Matrix3, Quaternion};
use rustc_hash::FxHashMap;
use threadpool::ThreadPool;

use ships::{Ship, ShipLoader};
use planet::Planet;

const WINDOW_TITLE: &str = "Starfarer";
const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;
const LOOK_SENSITIVITY: f32 = 0.1;
const NUM_SHADERS: usize = 100;
const MOVE_SENSITIVITY: f32 = 100.0;
const G: f64 = 1.5e0;

struct Starfarer {
    low_poly_shader: Shader<builtin::LPSignature>,
    ui_shader: Shader<builtin::UISignature>,

    camera: Camera,
    skybox: Skybox,
    lights: Lights,
    key_tracker: KeyTracker,
    last_deltas: (f64, f64),
    planet: Planet,
    threadpool: ThreadPool,

    ships: Vec<Ship>,
    sun: Object,

    player: Object,
    control_ship: Option<usize>,
    
    fps_menu: UserInterface<menus::Fps>,
    escape_menu: UserInterface<menus::Escape>,
    set_cursor_visible: bool,
}

impl Starfarer {
    fn new(graphics: &mut Graphics) -> Self {
        let low_poly_shader = Shader::new(graphics);
        let ui_shader = Shader::new(graphics);
        let camera = Camera::new(graphics, Vector3::new(2.0, 0.0, 1.0));
        let lights = Lights::new(graphics);
        let menu_common = menus::Common::new(graphics, &ui_shader);
        let fps_menu = menus::Fps::new(&menu_common);
        let escape_menu = menus::Escape::new(&menu_common);
        let mut object_manager = ObjectManager::new();
        let mut ship_loader = ShipLoader::new();
        let planet = Planet::new(0, 1_000.0, &mut object_manager);

        let ships = vec![
            ships::Ship::load(graphics, &low_poly_shader, &mut object_manager, &mut ship_loader, ships::compiled::enterprise::KESTREL,
                Vector3::new(20.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0), Quaternion::new(1.0, 0.01, -0.02, 0.03), Vector3::zero()),
            // ships::Ship::load(graphics, &low_poly_shader, &mut object_manager, &mut ship_loader, ships::compiled::enterprise::KESTREL,
            //     Vector3::new(8.0, 0.0, 0.0), Vector3::new(0.0, -40.0, 0.0), Quaternion::new(0.707, -0.001, 0.707, 0.001), Vector3::new(0.0, 0.4, 0.0)),
            // ships::Ship::load(graphics, &low_poly_shader, &mut object_manager, &mut ship_loader, ships::compiled::test::CUBE,
            //     Vector3::new(2.0, -2.0, 0.001), Vector3::new(0.0, -38.0, 0.0), Quaternion::new(1.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0)),
            ships::Ship::load(graphics, &low_poly_shader, &mut object_manager, &mut ship_loader, ships::compiled::test::CUBE,
                Vector3::new(30.0, 0.0, 0.0), Vector3::new(10.0, 0.0, 0.0), Quaternion::new(1.0, 0.71, -0.02, 0.3), Vector3::zero()),
        ];
        let player = object_manager.get_object();
        let sun = object_manager.get_object();
        let skybox = Skybox::from_temp(graphics);
        let threadpool = ThreadPool::new(8);

        Self {
            low_poly_shader,
            ui_shader,

            camera,
            skybox,
            lights,
            key_tracker: KeyTracker::new(),
            last_deltas: (0.0, 0.0),
            planet,
            threadpool,

            ships,
            sun,

            fps_menu,
            escape_menu,
            set_cursor_visible: false,

            player,
            control_ship: Some(0),
        }
    }

    fn control_character(&self, delta_time: f32, tasks: &mut Vec<PhysicsTask>) {
        let mut player_force = 
            - Vector3::unit_x() * ((self.key_tracker.get_state(VirtualKeyCode::W) as u32) as f32)
            + Vector3::unit_x() * ((self.key_tracker.get_state(VirtualKeyCode::S) as u32) as f32)
            - Vector3::unit_y() * ((self.key_tracker.get_state(VirtualKeyCode::A) as u32) as f32)
            + Vector3::unit_y() * ((self.key_tracker.get_state(VirtualKeyCode::D) as u32) as f32)
            + Vector3::unit_z() * ((self.key_tracker.get_state(VirtualKeyCode::LShift) as u32) as f32)
            - Vector3::unit_z() * ((self.key_tracker.get_state(VirtualKeyCode::LControl) as u32) as f32);
        if player_force.magnitude() > 0.0 {
            player_force *= delta_time * MOVE_SENSITIVITY / player_force.magnitude();
        }
        tasks.push(PhysicsTask::AddGlobalImpulse(self.player, (self.camera.get_rotation() * player_force).cast().unwrap()));
    }

    fn control_ship(&mut self, delta_time: f32, ship_index: usize, tasks: &mut Vec<PhysicsTask>) {
        let ship = &mut self.ships[ship_index];
        ship.continuous_commands(delta_time, &self.key_tracker);
        ship.poll_tasks(tasks);
    }
}

impl InputReceiver for Starfarer {
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
            self.last_deltas.0 += delta.0;
            self.last_deltas.1 += delta.1;
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
}

impl Renderer for Starfarer {
    fn interaction(tasks: &mut Vec<PhysicsTask>, (o_i, rb_i): (&Object, &RigidBody), (o_j, rb_j): (&Object, &RigidBody)) {
        if rb_i.mass > 500_000.0 {
            let dist = rb_j.pos - rb_i.pos;
            let force = -G * rb_i.mass * rb_j.mass / dist.magnitude2() * dist.normalize();
            tasks.push(PhysicsTask::AddGlobalForce(*o_j, force));
        } else if rb_j.mass > 500_000.0 {
            let dist = rb_i.pos - rb_j.pos;
            let force = -G * rb_i.mass * rb_j.mass / dist.magnitude2() * dist.normalize();
            tasks.push(PhysicsTask::AddGlobalForce(*o_i, force));
        }
    }

    fn load_models(&mut self, _graphics: &Graphics) -> FxHashMap<Object, Vec<DrawState>> {
        let mut map = FxHashMap::default();
        for ship in self.ships.iter_mut() {
            map.insert(ship.object, ship.get_models());
        }
        map.insert(self.sun, Vec::new());
        map.insert(self.player, Vec::new());
        map
    }

    fn load_rigid_bodies(&mut self) -> FxHashMap<Object, RigidBody> {
        let mut map = FxHashMap::default();
        for ship in self.ships.iter_mut() {
            map.insert(ship.object, ship.rigid_body.take().expect("Ship was created incorrectly or double loaded"));
        }
        map.insert(self.sun, RigidBody::by_pos(Vector3::new(10.0, 10.0, 10.0)));
        map.insert(self.player, RigidBody::new(
            Vector3::new(2.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0))
            .motivate(65.0, Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)));
        self.planet.init_rigid_body(&mut map);
        map
    }
    
    fn prepare(&mut self, _graphics: &Graphics) {
        self.lights.illuminate(self.sun, LightFeatures { diffuse_coeff: 0.5, specular_coeff: 1.0, shininess: 2, brightness: 0.5});
    }

    fn update(&mut self, graphics: &Graphics, delta_time: f32) -> Vec<PhysicsTask> {
        self.camera.turn(-self.last_deltas.1 as f32 * LOOK_SENSITIVITY * delta_time, -self.last_deltas.0 as f32 * LOOK_SENSITIVITY * delta_time);
        self.last_deltas = (0.0, 0.0);

        let mut tasks = Vec::new();

        match self.control_ship {
            Some(ship_index) => self.control_ship(delta_time, ship_index, &mut tasks),
            None => self.control_character(delta_time, &mut tasks),
        };

        self.planet.update(graphics, &self.low_poly_shader, &self.threadpool, self.camera.get_pos());

        self.fps_menu.data.update(delta_time, &mut self.fps_menu.elements);

        tasks
    }
    
    fn render(&mut self, graphics: &Graphics, buffer_index: usize) -> Vec<RenderTask> {
        if self.set_cursor_visible {
            graphics.set_cursor_visible(self.escape_menu.data.is_open);
        }
        // Update inputs
        if let Some(i) = self.control_ship {
            if let Some(data) = graphics.get_pos_and_rot(&self.ships[i].object) {
                self.camera.set_pos(data.0 + data.1 * self.ships[i].seat_pos);
                self.camera.set_local_rot(data.1);
            }
        } else if let Some(p) = graphics.get_pos(&self.player) {
            self.camera.set_pos(p);
        }
        self.camera.update_input(buffer_index);
        self.lights.update_input(graphics, buffer_index);

        let mut tasks = vec![
            RenderTask::LoadShader(&self.skybox.skybox_shader),
            RenderTask::DrawModel(&self.skybox.model),
            RenderTask::ClearDepthBuffer,
            RenderTask::LoadShader(&self.low_poly_shader),
        ];
        for ship in &self.ships {
            tasks.push(RenderTask::DrawObject(ship.object));
        }
        self.planet.render(&mut tasks);

        tasks.push(RenderTask::LoadShader(&self.ui_shader));
        tasks.push(RenderTask::DrawUI(&self.fps_menu));

        if self.escape_menu.data.is_open {
            tasks.push(RenderTask::DrawUI(&self.escape_menu));
        }

        tasks
    }

    fn should_quit(&self) -> bool {
        self.escape_menu.data.quit
    }
}

fn main() {
    let mut backend = Backend::new();
    let mut graphics = Graphics::new(&mut backend, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, true,
        vec![InputType::Camera, InputType::Lights, InputType::UI], NUM_SHADERS);
    
    let starfarer = Starfarer::new(&mut graphics);
    
    backend.run(graphics, starfarer);
}