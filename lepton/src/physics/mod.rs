mod rigid_body;

use std::sync::mpsc::{Receiver, Sender};
use std::collections::HashMap;
use cgmath::{Vector3};

pub use rigid_body::*;
use crate::backend::{Backend, ThreadData};
use crate::GraphicsData;

pub(crate) type PhysicsData = Vector3<f64>;
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

enum Updater {
    Fixed,
    Free,
    Line,
    Circle,
    Orbit,
}

enum Collider {
    None,
    Box,
    Sphere,
    Plane,
    BoundedPlane,
}

pub(crate) struct Physics {
    physics_data_receiver: Receiver<ThreadData<PhysicsData>>,
    graphics_data_sender: Sender<ThreadData<GraphicsData>>,

    pub(crate) rigid_bodies: HashMap<Object, RigidBody>,
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
            rigid_bodies: HashMap::new(),
        }
    }

    pub(crate) fn add_body(&mut self, object: Object, body: RigidBody) {
        self.rigid_bodies.insert(object, body);
    }

    pub(crate) fn update(&mut self, delta_time: f32) {

        // Detect collisions

        // Resolve collisions

        // Update body position
        for (_, body) in &mut self.rigid_bodies {
            body.update(delta_time as f64);
        }

        // Send data to graphics
        let mut graphics_data = HashMap::with_capacity(self.rigid_bodies.len());
        for (object, body) in &self.rigid_bodies {
            let pos = body.get_pos();
            graphics_data.insert(*object, GraphicsData {
                push_constants: body.push_constants(),
                pos: Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
            });
        }

        self.graphics_data_sender.send(graphics_data).unwrap();
    }
}