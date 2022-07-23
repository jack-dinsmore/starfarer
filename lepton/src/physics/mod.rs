mod rigid_body;
mod collisions;

use std::sync::mpsc::{Receiver, Sender};
use rustc_hash::FxHashMap;
use cgmath::{Vector3, InnerSpace};

pub use rigid_body::*;
pub use collisions::*;
use crate::backend::{Backend};
use crate::graphics::{GraphicsData, GraphicsInnerData};

pub(crate) type PhysicsData = Vec<PhysicsTask>;
pub type Object = u16;


pub struct ObjectManager {
    max_id: Object,
}

impl ObjectManager {
    pub fn new() -> Self {
        ObjectManager { max_id: 0 }
    }

    pub fn get_object(&mut self) -> Object {
        let id = self.max_id;
        if self.max_id == Object::MAX {
            panic!("Too many objects have been created");
        }
        self.max_id += 1;
        id
    }
}

pub enum Updater {
    Fixed,
    Free,
    Line,
    Circle,
    Orbit,
}

pub enum PhysicsTask {
    AddGlobalForce(Object, Vector3<f64>),
    AddGlobalImpulse(Object, Vector3<f64>),
    AddLocalForce(Object, Vector3<f64>),
    AddLocalImpulse(Object, Vector3<f64>),
    AddLocalImpulseTorque(Object, Vector3<f64>),
    AddGlobalImpulseTorque(Object, Vector3<f64>),
    ShiftPos(Object, Vector3<f64>),
}

pub(crate) struct Physics {
    physics_data_receiver: Receiver<PhysicsData>,
    physics_data_sender: Sender<PhysicsData>,
    graphics_data_sender: Sender<GraphicsData>,

    pub(crate) rigid_bodies: FxHashMap<Object, RigidBody>,
}

impl Physics {
    pub fn new(backend: &mut Backend) -> Physics {
        let physics_data_receiver = match backend.physics_data_receiver.take() {
            Some(r) => r,
            None => panic!("Someone picked up the physics data receiver")
        };
        let physics_data_sender = backend.physics_data_sender.clone();
        let graphics_data_sender = backend.graphics_data_sender.clone();

        Physics {
            physics_data_receiver,
            physics_data_sender,
            graphics_data_sender,
            rigid_bodies: FxHashMap::default(),
        }
    }

    pub(crate) fn add_body(&mut self, object: Object, body: RigidBody) {
        self.rigid_bodies.insert(object, body);
    }

    pub(crate) fn update(&mut self, delta_time: f32) {
        // Detect collisions
        let mut collision_results = Vec::new();
        for (i, (o_i, rb_i)) in self.rigid_bodies.iter().enumerate() {
            for (j, (o_j, rb_j)) in self.rigid_bodies.iter().enumerate() {
                if j <= i {
                    continue;
                }
                if let Some((n, r)) = rb_i.detect_collision_dist(&rb_j) {
                    // Fix position
                    let frac = match rb_i.updater {
                        Updater::Fixed => 0.0,
                        _ => match rb_j.updater {
                            Updater::Fixed => 1.0,
                            _ => rb_i.mass / (rb_i.mass + rb_j.mass),
                        },
                    };
                    
                    // Impulse
                    let r1 = r;
                    let r2 = r - (rb_j.pos - rb_i.pos);
                    let rel_vel = rb_j.vel - rb_i.vel;
                    let elasticity = (rb_i.elasticity * rb_j.elasticity).sqrt();
                    let normal = n.normalize();
                    let impulse = -(1.0 + elasticity) * normal.dot(rel_vel) * normal / (
                        1.0 / rb_i.mass + 1.0 / rb_j.mass + normal.dot(
                            rb_i.moi_inv * (r1.cross(normal).cross(r1))
                            + rb_j.moi_inv * (r2.cross(normal).cross(r2))
                        )
                    );
                    collision_results.push(PhysicsTask::AddGlobalImpulse(*o_i, -impulse));
                    collision_results.push(PhysicsTask::AddGlobalImpulse(*o_j, impulse));
                    collision_results.push(PhysicsTask::AddGlobalImpulseTorque(*o_i, -r1.cross(impulse)));
                    collision_results.push(PhysicsTask::AddGlobalImpulseTorque(*o_j, r2.cross(impulse)));
                    collision_results.push(PhysicsTask::ShiftPos(*o_i, -n * frac));
                    collision_results.push(PhysicsTask::ShiftPos(*o_j, n * (1.0 - frac)));
                }
            }
        }
        self.physics_data_sender.send(collision_results).unwrap();

        // Add forces
        for task_vec in self.physics_data_receiver.try_iter() {
            for task in task_vec.into_iter() {
                match task {
                    PhysicsTask::AddGlobalForce(object, force) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.force += force;
                        }
                    },
                    PhysicsTask::AddGlobalImpulse(object, impulse) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.force += impulse / delta_time as f64;
                        }
                    },
                    PhysicsTask::AddLocalForce(object, force) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.force += rb.orientation * force;
                        }
                    },
                    PhysicsTask::AddLocalImpulse(object, impulse) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.force += rb.orientation * impulse / delta_time as f64;
                        }
                    },
                    PhysicsTask::AddLocalImpulseTorque(object, impulse) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.torque += rb.orientation * impulse / delta_time as f64;
                        }
                    },
                    PhysicsTask::AddGlobalImpulseTorque(object, impulse) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.torque += impulse / delta_time as f64;
                        }
                    },
                    PhysicsTask::ShiftPos(object, delta) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.pos += delta;
                        }
                    }
                }
            }
        }

        // Resolve collisions

        // Update body position
        for body in self.rigid_bodies.values_mut() {
            body.update(delta_time as f64);
        }

        // Send data to graphics
        let mut graphics_data = FxHashMap::default();//with_capacity(self.rigid_bodies.len());
        for (object, body) in &self.rigid_bodies {
            let pos = body.get_pos();
            graphics_data.insert(*object, GraphicsInnerData {
                push_constants: body.push_constants(),
                pos: Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
            });
        }

        self.graphics_data_sender.send(graphics_data).unwrap_or(());
    }
}