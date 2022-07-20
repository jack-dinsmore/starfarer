#![allow(dead_code)]
mod primitives;

mod bytecode;
mod part;
use lepton::prelude::*;
use cgmath::{Vector3, Quaternion, InnerSpace, Zero};
use lepton::prelude::vertex::VertexLP;
use std::collections::HashMap;
use std::rc::Rc;
use starfarer_macros::include_model;

use part::*;
use primitives::*;
pub use primitives::compiled;

pub struct ShipLoader {
    models: HashMap<PartID, HashMap<String, Rc<Model>>>,
    paths: HashMap<PartID, String>,
}

impl ShipLoader {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    pub fn purge(&mut self) {
        self.models.clear();
    }
}

impl ShipLoader {
    /// Returns None if the model is known not to exist, and Some if the model does exist. Loads the model if it has not been loaded.
    fn acquire_models(&mut self, graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>, part_id: PartID) -> &HashMap<String, Rc<Model>> {
        if self.models.contains_key(&part_id) {
            self.models.get(&part_id).unwrap()
        } else {
            let models = self.load_models(graphics, low_poly_shader, part_id);
            self.models.insert(part_id, models);
            self.models.get(&part_id).unwrap()
        }
    }

    fn load_models(&self, graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>, part_id: PartID) -> HashMap<String, Rc<Model>> {
        let mut output = HashMap::new();
        for (key, value) in self.load_model_data(part_id).into_iter() {
            output.insert(key, Rc::new(Model::new(graphics, low_poly_shader, VertexType::Specified(value.0, value.1), TextureType::None).unwrap()));
        }
        output
    }

    fn load_model_data(&self, part_id: PartID) -> HashMap<String, (Vec<VertexLP>, Vec<u32>)> {
        // Look for part id in the paths dict
        match self.paths.get(&part_id) {
            Some(_) => unimplemented!("Loading models from a file is not implemented"),
            None => match part_id { // Assume the data was compiled
                compiled::enterprise::KESTREL => bincode::deserialize(include_model!("../../assets/enterprise/kestrel")).unwrap(),
                _ => match MakeID::from(part_id) {
                    compiled::enterprise::MAKE => bincode::deserialize(include_model!("../../assets/enterprise/accessories")).unwrap(),
                    _ => panic!("Models of {:?} were not compiled", part_id)
                }
            }
        }
    }

    fn load_ship_data(&self, id: PartID) -> ShipData {
        // Look for part id in the paths dict
        match self.paths.get(&id) {
            Some(_) => unimplemented!("Loading ship data from a file is not implemented"),
            None => match id { // Assume the data was compiled
                compiled::enterprise::KESTREL => bincode::deserialize(include_bytes!("../../assets/enterprise/kestrel/kestrel.dat")).unwrap(),
                _ => panic!("Ship {:?} was not compiled.", id)
            }
        }
    }

    fn load_part_data(&self, id: PartID) -> PartData {
        match self.paths.get(&id) {
            Some(_) => unimplemented!("Loading ship data from a file is not implemented"),
            None => match id { // Assume the data was compiled
                compiled::enterprise::CHAIR => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/chair.dat")).unwrap(),
                compiled::enterprise::DISH => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/dish.dat")).unwrap(),
                compiled::enterprise::PORT => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/port.dat")).unwrap(),
                compiled::enterprise::RADIATOR => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/radiator.dat")).unwrap(),
                compiled::enterprise::RCS => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/rcs.dat")).unwrap(),
                compiled::enterprise::SOLAR => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/solar.dat")).unwrap(),
                _ => panic!("Part {:?} was not compiled.", id)
            }
        }
    }
}

pub struct Ship {
    pub object: Object,
    pub outside_model: Option<Rc<Model>>, // These are all options so they can be taken.
    pub inside_model: Option<Rc<Model>>,
    pub transparent_model: Option<Rc<Model>>,
    pub rigid_body: Option<RigidBody>,
    pub seat_pos: Vector3<f32>,
    attachments: Vec<PartState>,

    // For runtime
    tasks: Vec<PhysicsTask>,
}

impl Ship {
    pub fn load(graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>,
        object_manager: &mut ObjectManager, ship_loader: &mut ShipLoader, id: PartID,
        pos: Vector3<f64>, vel: Vector3<f64>, orientation: Quaternion<f64>, ang_vel: Vector3<f64>) -> Ship {

        let data = ship_loader.load_ship_data(id);
        let models = ship_loader.acquire_models(graphics, low_poly_shader, id);
        let mut outside_model = None;
        let mut inside_model = None;
        let mut transparent_model = None;
        for (name, model) in models {
            if name.starts_with("outside") {
                outside_model = Some(model.clone());
            } else if name.starts_with("inside") {
                inside_model = Some(model.clone());
            } else if name.starts_with("transparent") {
                transparent_model = Some(model.clone());
            }
        }

        let rigid_body = Some(RigidBody::new(pos, vel, orientation, ang_vel)
            .motivate(data.mass, data.moment_of_inertia)
            .offset(data.center_of_mass)
            .collide(Collider::cube(5.0), Vector3::zero())
        );
        let mut attachments = Vec::with_capacity(data.attachments.len());
        for attachment in data.attachments.into_iter() {
            let part_data = ship_loader.load_part_data(attachment.id);
            let model = match ship_loader.acquire_models(graphics, low_poly_shader, attachment.id).get(&part_data.object_name) {
                Some(m) => m,
                None => panic!("Part {:?} was not contained in attachments", attachment.id)
            };
            attachments.push(PartState::from_instance(attachment, model.clone()));
        }

        let object = object_manager.get_object();
        Ship {
            object,
            outside_model,
            inside_model,
            transparent_model,
            rigid_body,
            seat_pos: data.seat_pos,
            attachments,

            tasks: Vec::new(),
        }
    }

    pub fn get_models(&mut self) -> Vec<DrawState> {
        let mut output = Vec::new();
        match self.outside_model.take() {
            Some(o) => output.push(DrawState::Standard(o)),
            None => panic!("Ship had no outside model")
        };
        match self.inside_model.take() {
            Some(o) => output.push(DrawState::Standard(o)),
            None => panic!("Ship had no inside model")
        };
        match self.transparent_model.take() {
            Some(o) => output.push(DrawState::Standard(o)),
            None => ()
        };
        for attachment in self.attachments.iter_mut() {
            output.push(DrawState::Offset(
                attachment.model.take().expect("Attachment was double loaded"),
                attachment.matrix))
        }
        output
    }

    pub fn continuous_commands(&mut self, delta_time: f32, key_tracker: &KeyTracker) {
        let mut ship_force = 
              Vector3::unit_x() * ((key_tracker.get_state(VirtualKeyCode::Up) as u32) as f32)
            - Vector3::unit_x() * ((key_tracker.get_state(VirtualKeyCode::Down) as u32) as f32)
            + Vector3::unit_y() * ((key_tracker.get_state(VirtualKeyCode::Left) as u32) as f32)
            - Vector3::unit_y() * ((key_tracker.get_state(VirtualKeyCode::Right) as u32) as f32)
            + Vector3::unit_z() * ((key_tracker.get_state(VirtualKeyCode::RShift) as u32) as f32)
            - Vector3::unit_z() * ((key_tracker.get_state(VirtualKeyCode::RAlt) as u32) as f32);
        if ship_force.magnitude() > 0.0 {
            ship_force *= delta_time * 40_000.0 / ship_force.magnitude();
        }
        self.tasks.push(PhysicsTask::AddLocalImpulse(self.object, ship_force.cast().unwrap()));
        let mut ship_torque = 
              Vector3::unit_y() * ((key_tracker.get_state(VirtualKeyCode::W) as u32) as f32)
            - Vector3::unit_y() * ((key_tracker.get_state(VirtualKeyCode::S) as u32) as f32)
            + Vector3::unit_z() * ((key_tracker.get_state(VirtualKeyCode::A) as u32) as f32)
            - Vector3::unit_z() * ((key_tracker.get_state(VirtualKeyCode::D) as u32) as f32)
            - Vector3::unit_x() * ((key_tracker.get_state(VirtualKeyCode::Q) as u32) as f32)
            + Vector3::unit_x() * ((key_tracker.get_state(VirtualKeyCode::E) as u32) as f32);
        if ship_torque.magnitude() > 0.0 {
            ship_torque *= delta_time * 20_000.0 / ship_torque.magnitude();
        }
        self.tasks.push(PhysicsTask::AddLocalImpulseTorque(self.object, ship_torque.cast().unwrap()));
    }

    pub fn poll_tasks(&mut self, tasks: &mut Vec<PhysicsTask>) {
        tasks.append(&mut self.tasks);
    }
}
