mod rigid_body;

use std::sync::mpsc::{Receiver, Sender};
use rustc_hash::FxHashMap;
use cgmath::{Vector3};

pub use rigid_body::*;
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

pub enum Collider {
    None,
    Box,
    Sphere,
    Plane,
    BoundedPlane,
}

pub enum PhysicsTask {
    AddGlobalForce(Object, Vector3<f64>),
    AddGlobalImpulse(Object, Vector3<f64>),
    AddLocalForce(Object, Vector3<f64>),
    AddLocalImpulse(Object, Vector3<f64>),
    AddLocalImpulseTorque(Object, Vector3<f64>),
}

pub(crate) struct Physics {
    physics_data_receiver: Receiver<PhysicsData>,
    graphics_data_sender: Sender<GraphicsData>,

    pub(crate) rigid_bodies: FxHashMap<Object, RigidBody>,
}

impl Physics {
    pub fn new(backend: &mut Backend) -> Physics {
        let physics_data_receiver = match backend.physics_data_receiver.take() {
            Some(r) => r,
            None => panic!("Someone picked up the physics data receiver")
        };
        let graphics_data_sender = match backend.graphics_data_sender.take() {
            Some(r) => r,
            None => panic!("Someone picked up the graphics data sender")
        };

        Physics {
            physics_data_receiver,
            graphics_data_sender,
            rigid_bodies: FxHashMap::default(),
        }
    }

    pub(crate) fn add_body(&mut self, object: Object, body: RigidBody) {
        self.rigid_bodies.insert(object, body);
    }

    pub(crate) fn update(&mut self, delta_time: f32) {
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
                }
            }
        }

        // Detect collisions

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