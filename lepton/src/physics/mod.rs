mod rigid_body;
mod collider;

use std::sync::mpsc::{Receiver, Sender};
use rustc_hash::FxHashMap;
use cgmath::{Vector3, InnerSpace};

pub use rigid_body::*;
pub use collider::*;
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

pub(crate) struct Physics<F: Fn(&mut Vec<PhysicsTask>, (&Object, &RigidBody), (&Object, &RigidBody))> {
    physics_data_receiver: Receiver<PhysicsData>,
    physics_data_sender: Sender<PhysicsData>,
    graphics_data_sender: Sender<GraphicsData>,
    interaction: F,
    pub(crate) rigid_bodies: FxHashMap<Object, RigidBody>,
}

impl<F: Fn(&mut Vec<PhysicsTask>, (&Object, &RigidBody), (&Object, &RigidBody))> Physics<F> {
    pub fn new(backend: &mut Backend, interaction: F) -> Self {
        let physics_data_receiver = match backend.physics_data_receiver.take() {
            Some(r) => r,
            None => panic!("Someone picked up the physics data receiver")
        };
        let physics_data_sender = backend.physics_data_sender.clone();
        let graphics_data_sender = backend.graphics_data_sender.clone();

        Self {
            physics_data_receiver,
            physics_data_sender,
            graphics_data_sender,
            interaction,
            rigid_bodies: FxHashMap::default(),
        }
    }

    pub(crate) fn add_body(&mut self, object: Object, body: RigidBody) {
        self.rigid_bodies.insert(object, body);
    }

    pub(crate) fn update(&mut self, delta_time: f32) {
        let mut interaction_forces = Vec::new();
        // Detect and act on collisions
        for (o_i, rb_i) in self.rigid_bodies.iter() {
            for (o_j, rb_j) in self.rigid_bodies.iter() {
                if o_j <= o_i {
                    continue;
                }

                (self.interaction)(&mut interaction_forces, (o_i, rb_i), (o_j, rb_j));

                if let Some((n, r)) = rb_i.detect_collision_dist(&rb_j) {
                    let r1 = r;
                    let r2 = r - (rb_j.pos - rb_i.pos);
                    let rel_vel = rb_j.vel + rb_j.ang_vel.cross(r1) - rb_i.vel - rb_i.ang_vel.cross(r2);
                    let elasticity = (rb_i.elasticity * rb_j.elasticity).sqrt();
                    let normal = n.normalize();
                    let i_denom = match rb_i.updater {
                        Updater::Fixed => 0.0,
                        _ => 1.0 / rb_i.mass + normal.dot(rb_i.moi_inv * (r1.cross(normal).cross(r1)))
                    };
                    let j_denom = match rb_j.updater {
                        Updater::Fixed => 0.0,
                        _ => 1.0 / rb_j.mass + normal.dot(rb_j.moi_inv * (r2.cross(normal).cross(r2)))
                    };
                    let impulse = -(1.0 + elasticity) * normal.dot(rel_vel) * normal / (i_denom + j_denom);

                    let (rb_i, rb_j): (&mut RigidBody, &mut RigidBody) = unsafe {
                        // Convert to mutable references. This is safe because they are non-identical and I'm not editing the hash.
                        let i_ptr = rb_i as *const _ as *mut _;
                        let j_ptr = rb_j as *const _ as *mut _;
                        (&mut *i_ptr, &mut *j_ptr)
                    };

                    rb_i.impulse -= impulse;
                    rb_i.torque_impulse -= r1.cross(impulse);
                    rb_i.collide_normal = Some(normal);

                    rb_j.impulse += impulse;
                    rb_j.torque_impulse += r2.cross(impulse);
                    rb_j.collide_normal = Some(-normal);
                }
            }
        }

        self.physics_data_sender.send(interaction_forces).unwrap();

        // Add forces
        for task_vec in self.physics_data_receiver.try_iter() {
            for task in task_vec.into_iter() {
                match task {
                    PhysicsTask::AddGlobalForce(object, force) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            if let Some(n) = rb.collide_normal {
                                if force.dot(n) > 0.0 {
                                    continue;
                                }
                            }
                            rb.impulse += force * delta_time as f64;
                        }
                    },
                    PhysicsTask::AddGlobalImpulse(object, impulse) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            if let Some(n) = rb.collide_normal {
                                if impulse.dot(n) > 0.0 {
                                    continue;
                                }
                            }
                            rb.impulse += impulse;
                        }
                    },
                    PhysicsTask::AddLocalForce(object, force) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            let force = rb.orientation * force;
                            if let Some(n) = rb.collide_normal {
                                if force.dot(n) > 0.0 {
                                    continue;
                                }
                            }
                            rb.impulse += force * delta_time as f64;
                        }
                    },
                    PhysicsTask::AddLocalImpulse(object, impulse) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            let force = rb.orientation * impulse / delta_time as f64;
                            if let Some(n) = rb.collide_normal {
                                if force.dot(n) > 0.0 {
                                    continue;
                                }
                            }
                            rb.impulse += force * delta_time as f64;
                        }
                    },
                    PhysicsTask::AddLocalImpulseTorque(object, impulse) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.torque_impulse += rb.orientation * impulse;
                        }
                    },
                    PhysicsTask::AddGlobalImpulseTorque(object, impulse) => {
                        if let Some(rb) = self.rigid_bodies.get_mut(&object) {
                            rb.torque_impulse += impulse;
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