use cgmath::{Vector3, Matrix3, Quaternion, Matrix4, Matrix, Zero, InnerSpace, SquareMatrix, Rotation};

use crate::shader::builtin;
use super::{Updater, Collider, collider::{GJKState, CollisionType}};

pub struct RigidBody {
    pub pos: Vector3<f64>,
    pub vel: Vector3<f64>,
    pub mass: f64,
    pub impulse: Vector3<f64>,
    pub torque_impulse: Vector3<f64>,

    pub orientation: Quaternion<f64>,
    pub ang_vel: Vector3<f64>, // Local frame
    pub local_moi: Matrix3<f64>,
    pub local_moi_inv: Matrix3<f64>,
    
    pub updater: Updater,
    
    pub colliders: Vec<Collider>,
    pub elasticity: f64,
    pub model_offset: Matrix4<f32>,
    pub collide_normal: Option<Vector3<f64>>,
}

impl RigidBody {
    pub fn new(pos: Vector3<f64>, vel: Vector3<f64>, orientation: Quaternion<f64>, ang_vel: Vector3<f64>) -> Self {
        Self {
            pos,
            vel,
            mass: 0.0,
            impulse: Vector3::new(0.0, 0.0, 0.0),
            torque_impulse: Vector3::new(0.0, 0.0, 0.0),
            orientation,
            ang_vel,
            local_moi: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            local_moi_inv: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            updater: Updater::Fixed,
            colliders: Vec::new(),
            elasticity: 1.0,
            model_offset: Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0)),
            collide_normal: None,
        }
    }

    pub fn by_pos(pos: Vector3<f64>) -> Self {
        Self {
            pos,
            vel: Vector3::new(0.0, 0.0, 0.0),
            mass: 0.0,
            impulse: Vector3::new(0.0, 0.0, 0.0),
            torque_impulse: Vector3::new(0.0, 0.0, 0.0),
            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            ang_vel: Vector3::new(0.0, 0.0, 0.0),
            local_moi: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            local_moi_inv: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            updater: Updater::Fixed,
            colliders: Vec::new(),
            elasticity: 1.0,
            model_offset: Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0)),
            collide_normal: None,
        }
    }

    pub fn offset(mut self, model_offset: Vector3<f32>) -> Self {
        self.model_offset = Matrix4::from_translation(model_offset);
        self
    }

    pub fn motivate(mut self, mass: f64, moi: Matrix3<f32>) -> Self {
        self.local_moi = moi.cast().unwrap();
        self.local_moi_inv = self.local_moi.invert().unwrap();
        self.mass = mass;
        self.updater = Updater::Free;
        self
    }

    pub fn gravitate(mut self, mass: f64) -> Self {
        self.mass = mass;
        self
    }
    
    pub fn collide(mut self, colliders: Vec<Collider>, elasticity: f64) -> Self {
        self.colliders = colliders;
        self.elasticity = elasticity;
        self
    }

    pub fn moi(&self) -> Matrix3<f64> {
        let mat = Matrix3::from(self.orientation);
        mat * self.local_moi * mat.transpose()
    }

    pub fn moi_inv(&self) -> Matrix3<f64> {
        let mat = Matrix3::from(self.orientation);
        mat * self.local_moi_inv * mat.transpose()
    }

}

impl RigidBody {
    pub(crate) fn push_constants(&self) -> builtin::ObjectPushConstants {
        let rotation = Matrix4::from(Matrix3::from(self.orientation.cast().unwrap()));
        builtin::ObjectPushConstants {
            model: Matrix4::from_translation(self.pos.cast().unwrap()) * rotation * self.model_offset,
            rotation,
        }
    }

    pub(crate) fn get_pos(&self) -> Vector3<f64> {
        self.pos
    }

    pub(crate) fn update(&mut self, delta_time: f64) {
        match self.updater {
            Updater::Fixed => (),
            Updater::Free => {
                self.vel += self.impulse / self.mass;
                self.pos += self.vel * delta_time;
                self.ang_vel += self.moi_inv() * (self.torque_impulse - self.ang_vel.cross(self.moi() * self.ang_vel) * delta_time);
                self.orientation += 0.5 * Quaternion::new(0.0, self.ang_vel.x, self.ang_vel.y, self.ang_vel.z) * self.orientation * delta_time;
                self.orientation = self.orientation.normalize();
                self.impulse = Vector3::zero();
                self.torque_impulse = Vector3::zero();
                self.collide_normal = None;
            },
            _ => unimplemented!()
        }
        
    }

    pub(crate) fn detect_collision_dist(&self, o: &RigidBody, shift: Vector3<f64>) -> Option<(Vector3<f64>, Vector3<f64>)> {
        // Work in inertial frame, self-centric. Shift indicates the velocity of the other object. 

        for my_c in &self.colliders {
            for o_c in &o.colliders {
                // Confirm the objects are within collision distance
                let my_c_offset = self.orientation * my_c.offset();
                let o_c_offset = o.orientation * o_c.offset();
                let displacement = o.pos - self.pos - my_c_offset + o_c_offset;
                if displacement.magnitude() > my_c.radius() + o_c.radius() {
                    continue;
                }

                // Initialization
                let my_inv_orientation = self.orientation.invert();
                let o_inv_orientation = o.orientation.invert();
                let initial_axis = displacement.normalize();

                let my_pos = my_c.support(my_inv_orientation * initial_axis);
                let o_pos = o_c.support(o_inv_orientation * -initial_axis);
                let mut dir = -(self.orientation * my_pos - (o.orientation * o_pos + displacement));
                if shift.dot(initial_axis) > 0.0 {
                    dir -= shift;
                }
                let mut state = GJKState::new((-dir, my_pos, o_pos));
                let collision = loop {
                    dir = dir.normalize();
                    let my_pos = my_c.support(my_inv_orientation * dir);
                    let o_pos = o_c.support(o_inv_orientation * -dir);
                    let mut new_vec = self.orientation * my_pos - (o.orientation * o_pos + displacement);
                    if shift.dot(dir) > 0.0 {
                        new_vec += shift;
                    }

                    if new_vec.dot(dir) < 0.0 {
                        break None;
                    }
                    state.push((new_vec, my_pos, o_pos));
                    if match state.contains_origin(&mut dir) {
                        Err(_) => break None,
                        Ok(b) => b
                    } {
                        // Process collision
                        let (p1, p2, center, mut normal) = match state.get_collision_type() {
                            CollisionType::FaceVertex((v0, v1, v2), (o0,)) => {
                                let v0 = self.orientation * v0 + my_c_offset;
                                let v1 = self.orientation * v1 + my_c_offset;
                                let v2 = self.orientation * v2 + my_c_offset;
                                let o0 = o.orientation * o0 + my_c_offset - o_c_offset + o.pos - self.pos;
                                (v0, o0, o0, (v1 - v0).cross(v2 - v0))
                            },
                            CollisionType::VertexFace((v0,), (o0, o1, o2)) => {
                                let v0 = self.orientation * v0 + my_c_offset;
                                let o0 = o.orientation * o0 + my_c_offset - o_c_offset + o.pos - self.pos;
                                let o1 = o.orientation * o1 + my_c_offset - o_c_offset + o.pos - self.pos;
                                let o2 = o.orientation * o2 + my_c_offset - o_c_offset + o.pos - self.pos;
                                (v0, o0, v0, (o1 - o0).cross(o2 - o0))
                            },
                            CollisionType::EdgeEdge((v0, v1), (o0, o1)) => {
                                let v0 = self.orientation * v0 + my_c_offset;
                                let v1 = self.orientation * v1 + my_c_offset;
                                let o0 = o.orientation * o0 + my_c_offset - o_c_offset + o.pos - self.pos;
                                let o1 = o.orientation * o1 + my_c_offset - o_c_offset + o.pos - self.pos;
                                (v0, o0, (v0 + o0 + v1 + o1) / 4.0, (v1 - v0).cross(o1 - o0))
                            },
                            CollisionType::Other => {
                                break None;
                            },
                        };

                        normal *= normal.dot(p2 - p1) / normal.magnitude2();
                        break Some((normal, center));
                    }
                };
                if let Some(c) = collision {
                    return Some(c);
                }
            }
        }
        None
    }
}