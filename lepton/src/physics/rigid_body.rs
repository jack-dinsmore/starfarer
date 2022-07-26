use cgmath::{Vector3, Matrix3, Quaternion, Matrix4, Zero, InnerSpace, SquareMatrix, Rotation};

use crate::shader::builtin;
use super::{Updater, Collider, collisions::{GJKState, CollisionType}};

pub struct RigidBody {
    pub(super) pos: Vector3<f64>,
    pub(super) vel: Vector3<f64>,
    pub(super) mass: f64,
    pub(super) force: Vector3<f64>,
    pub(super) torque: Vector3<f64>,

    pub(super) orientation: Quaternion<f64>,
    pub(super) ang_vel: Vector3<f64>, // Local frame
    pub(super) moi: Matrix3<f64>,
    pub(super) moi_inv: Matrix3<f64>,
    
    pub(super) updater: Updater,
    
    pub(super) collider: Collider,
    pub(super) elasticity: f64,
    pub(super) collider_offset: Vector3<f64>,
    pub(super) model_offset: Vector3<f32>,
    pub(super) collide_normal: Option<Vector3<f64>>,
}

impl RigidBody {
    pub fn new(pos: Vector3<f64>, vel: Vector3<f64>, orientation: Quaternion<f64>, ang_vel: Vector3<f64>) -> Self {
        Self {
            pos,
            vel,
            mass: 0.0,
            force: Vector3::new(0.0, 0.0, 0.0),
            torque: Vector3::new(0.0, 0.0, 0.0),
            orientation,
            ang_vel,
            moi: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            moi_inv: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            updater: Updater::Fixed,
            collider: Collider::None,
            elasticity: 1.0,
            collider_offset: Vector3::zero(),
            model_offset: Vector3::new(0.0, 0.0, 0.0),
            collide_normal: None,
        }
    }

    pub fn by_pos(pos: Vector3<f64>) -> Self {
        Self {
            pos,
            vel: Vector3::new(0.0, 0.0, 0.0),
            mass: 0.0,
            force: Vector3::new(0.0, 0.0, 0.0),
            torque: Vector3::new(0.0, 0.0, 0.0),
            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            ang_vel: Vector3::new(0.0, 0.0, 0.0),
            moi: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            moi_inv: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            updater: Updater::Fixed,
            collider: Collider::None,
            elasticity: 1.0,
            collider_offset: Vector3::zero(),
            model_offset: Vector3::new(0.0, 0.0, 0.0),
            collide_normal: None,
        }
    }

    pub fn offset(mut self, model_offset: Vector3<f32>) -> Self {
        self.model_offset = model_offset;
        self
    }

    pub fn motivate(mut self, mass: f64, moi: Matrix3<f32>) -> Self {
        self.moi = moi.cast().unwrap();
        self.moi_inv = self.moi.invert().unwrap();
        self.mass = mass;
        self.updater = Updater::Free;
        self
    }
    
    pub fn collide(mut self, collider: Collider, elasticity: f64, collider_offset: Vector3<f64>) -> Self {
        self.collider = collider;
        self.elasticity = elasticity;
        self.collider_offset = collider_offset;
        self
    }

}

impl RigidBody {
    pub(crate) fn push_constants(&self) -> builtin::ObjectPushConstants {
        let rotation = Matrix4::from(Matrix3::from(self.orientation.cast().unwrap()));
        builtin::ObjectPushConstants {
            model: Matrix4::from_translation(self.pos.cast().unwrap() + self.model_offset) * rotation,
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
                self.vel += self.force / self.mass * delta_time;
                self.pos += self.vel * delta_time;
                self.ang_vel += self.moi_inv * (self.torque - self.ang_vel.cross(self.moi * self.ang_vel)) * delta_time;
                self.orientation += 0.5 * Quaternion::new(0.0, self.ang_vel.x, self.ang_vel.y, self.ang_vel.z) * self.orientation * delta_time;
                self.orientation = self.orientation.normalize();
                self.force = Vector3::zero();
                self.torque = Vector3::zero();
                self.collide_normal = None;
            },
            _ => unimplemented!()
        }
        
    }

    pub(crate) fn detect_collision_dist(&self, o: &RigidBody) -> Option<(Vector3<f64>, Vector3<f64>)> {
        // Work in axis-aligned frame, self-centric.

        // Confirm both colliders are active
        if let Collider::None = self.collider {
            return None;
        }
        if let Collider::None = o.collider {
            return None;
        }

        // Confirm the objects are within collision distance
        let displacement = o.pos - self.pos - self.collider_offset + o.collider_offset;
        if displacement.magnitude() > self.collider.radius() + o.collider.radius() {
            return None;
        }
        
        // Initialization
        let my_inv_orientation = self.orientation.invert();
        let o_inv_orientation = o.orientation.invert();
        let initial_axis = Vector3::new(1.0, 0.1, 0.06).normalize();

        let (my_id, my_pos) = self.collider.support(my_inv_orientation * initial_axis);
        let (o_id, o_pos) = o.collider.support(o_inv_orientation * -initial_axis);
        let mut dir = -(self.orientation * my_pos - (o.orientation * o_pos + displacement));
        let mut state = GJKState::new((-dir, my_id, o_id));
        loop {
            dir = dir.normalize();
            let (my_id, my_pos) = self.collider.support(my_inv_orientation * dir);
            let (o_id, o_pos) = o.collider.support(o_inv_orientation * -dir);
            let new_vec = self.orientation * my_pos - (o.orientation * o_pos + displacement);

            if new_vec.dot(dir) < 0.0 {
                return None;
            }
            state.push((new_vec, my_id, o_id));
            if match state.contains_origin(&mut dir) {
                Err(_) => return None,
                Ok(b) => b
            } {
                // Process collision
                let (p1, p2, center, mut normal) = match state.get_collision_type() {
                    Ok(t) => match t{
                        CollisionType::FaceVertex((v0, v1, v2), (o0,)) => {
                            let v0 = self.orientation * self.collider.id_to_vertex(v0) + self.collider_offset;
                            let v1 = self.orientation * self.collider.id_to_vertex(v1) + self.collider_offset;
                            let v2 = self.orientation * self.collider.id_to_vertex(v2) + self.collider_offset;
                            let o0 = o.orientation * o.collider.id_to_vertex(o0) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            (v0, o0, o0, (v1 - v0).cross(v2 - v0))
                        },
                        CollisionType::VertexFace((v0,), (o0, o1, o2)) => {
                            let v0 = self.orientation * self.collider.id_to_vertex(v0) + self.collider_offset;
                            let o0 = o.orientation * o.collider.id_to_vertex(o0) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            let o1 = o.orientation * o.collider.id_to_vertex(o1) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            let o2 = o.orientation * o.collider.id_to_vertex(o2) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            (v0, o0, v0, (o1 - o0).cross(o2 - o0))
                        },
                        CollisionType::EdgeEdge((v0, v1), (o0, o1)) => {
                            let v0 = self.orientation * self.collider.id_to_vertex(v0) + self.collider_offset;
                            let v1 = self.orientation * self.collider.id_to_vertex(v1) + self.collider_offset;
                            let o0 = o.orientation * o.collider.id_to_vertex(o0) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            let o1 = o.orientation * o.collider.id_to_vertex(o1) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            (v0, o0, (v0 + o0 + v1 + o1) / 4.0, (v1 - v0).cross(o1 - o0))
                        },
                        CollisionType::FaceFace((v0, v1, v2), (o0, o1, o2)) => {
                            let v0 = self.orientation * self.collider.id_to_vertex(v0) + self.collider_offset;
                            let v1 = self.orientation * self.collider.id_to_vertex(v1) + self.collider_offset;
                            let v2 = self.orientation * self.collider.id_to_vertex(v2) + self.collider_offset;
                            let o0 = o.orientation * o.collider.id_to_vertex(o0) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            let o1 = o.orientation * o.collider.id_to_vertex(o1) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            let o2 = o.orientation * o.collider.id_to_vertex(o2) + self.collider_offset - o.collider_offset + o.pos - self.pos;
                            (v0, o0, (v0 + v1 + v2 + o0 + o1 + o2) / 6.0, (o1 - o0).cross(o2 - o0) + (v1 - v0).cross(v2 - v0))
                        },
                    },
                    Err(e) => {
                        println!("Could not resolve collison because {:?}", e);
                        return None;
                    }
                };

                normal *= normal.dot(p2 - p1) / normal.magnitude2();
                return Some((normal, center));
            }
        }
    }
}