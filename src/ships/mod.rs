#![allow(dead_code)]
mod primitives;
mod loader;

mod bytecode;
mod part;
use lepton::prelude::*;
use cgmath::{Vector3, Quaternion, InnerSpace};

use part::*;
use primitives::*;
pub use loader::ShipLoader;
pub use primitives::compiled;

pub struct Ship {
    pub object: Object,
    id: PartID,
    pub seat_pos: Vector3<f32>,
    attachments: Vec<PartState>,

    // For runtime
    tasks: Vec<PhysicsTask>,
}

impl Ship {
    pub fn new(object_manager: &mut ObjectManager, ship_loader: &mut ShipLoader, id: PartID) -> Ship {
        let data = ship_loader.load_ship_data(id);
        let mut attachments = Vec::with_capacity(data.attachments.len());
        for attachment in data.attachments.into_iter() {
            attachments.push(PartState::from_instance(attachment));
        }

        let object = object_manager.get_object();
        Ship {
            object,
            id,
            seat_pos: data.seat_pos,
            attachments,

            tasks: Vec::new(),
        }
    }

    pub fn get_models(&self, graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>, ship_loader: &mut ShipLoader) -> Vec<DrawState> {
        let mut output = Vec::new();
        let models = ship_loader.acquire_models(graphics, low_poly_shader, self.id);
        let mut outside_model = None;
        let mut inside_model = None;

        for (name, model) in models {
            if name.starts_with("outside") {
                outside_model = Some(DrawState::Standard(model.clone()));
            } else if name.starts_with("inside") {
                inside_model = Some(DrawState::Standard(model.clone()));
            } else if name.starts_with("transparent") {
                output.push(DrawState::Standard(model.clone()));
            }
        }

        output.insert(0, outside_model.take().expect("No outside model was provided"));
        output.insert(0, inside_model.take().expect("No inside model was provided"));

        for attachment in self.attachments.iter() {
            let part_data = ship_loader.load_part_data(attachment.id);
            let model = match ship_loader.acquire_models(graphics, low_poly_shader, attachment.id).get(&part_data.object_name) {
                Some(m) => m,
                None => panic!("Part {:?} was not contained in attachments", attachment.id)
            };
            output.push(DrawState::Offset(model.clone(), attachment.matrix));
        }

        output
    }

    pub fn get_rigid_body(&self, ship_loader: &mut ShipLoader, pos: Vector3<f64>, vel: Vector3<f64>, orientation: Quaternion<f64>, ang_vel: Vector3<f64>) -> RigidBody {
        let data = ship_loader.load_ship_data(self.id);
        RigidBody::new(pos, vel, orientation, ang_vel)
            .motivate(data.mass, data.moment_of_inertia)
            .offset(-data.center_of_mass)
            .collide(ship_loader.load_colliders(data.id), data.elasticity)
    }

    pub fn continuous_commands(&mut self, delta_time: f32, key_tracker: &KeyTracker) {
        let mut ship_force = 
              Vector3::unit_x() * ((key_tracker.get_state(VirtualKeyCode::Up) as u32) as f32)
            - Vector3::unit_x() * ((key_tracker.get_state(VirtualKeyCode::Down) as u32) as f32)
            + Vector3::unit_y() * ((key_tracker.get_state(VirtualKeyCode::Left) as u32) as f32)
            - Vector3::unit_y() * ((key_tracker.get_state(VirtualKeyCode::Right) as u32) as f32)
            + Vector3::unit_z() * ((key_tracker.get_state(VirtualKeyCode::RShift) as u32) as f32)
            - Vector3::unit_z() * ((key_tracker.get_state(VirtualKeyCode::RAlt) as u32) as f32);
        if ship_force.magnitude() > 0.1 {
            ship_force *= delta_time * 80_000.0 / ship_force.magnitude();
            self.tasks.push(PhysicsTask::AddLocalImpulse(self.object, ship_force.cast().unwrap()));
        }
        let mut ship_torque = 
              Vector3::unit_y() * ((key_tracker.get_state(VirtualKeyCode::W) as u32) as f32)
            - Vector3::unit_y() * ((key_tracker.get_state(VirtualKeyCode::S) as u32) as f32)
            + Vector3::unit_z() * ((key_tracker.get_state(VirtualKeyCode::A) as u32) as f32)
            - Vector3::unit_z() * ((key_tracker.get_state(VirtualKeyCode::D) as u32) as f32)
            - Vector3::unit_x() * ((key_tracker.get_state(VirtualKeyCode::Q) as u32) as f32)
            + Vector3::unit_x() * ((key_tracker.get_state(VirtualKeyCode::E) as u32) as f32);
        if ship_torque.magnitude() > 0.1 {
            ship_torque *= delta_time * 200_000.0 / ship_torque.magnitude();
            self.tasks.push(PhysicsTask::AddLocalImpulseTorque(self.object, ship_torque.cast().unwrap()));
        }
    }

    pub fn poll_tasks(&mut self, tasks: &mut Vec<PhysicsTask>) {
        tasks.append(&mut self.tasks);
    }
}
