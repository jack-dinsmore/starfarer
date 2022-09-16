use cgmath::{Vector3, Matrix3, Quaternion, Matrix4, Matrix, Zero, InnerSpace, SquareMatrix, Rotation};

use crate::shader::builtin;
use super::{Updater, Collider, collider::{GJKState, CollisionData}};

const COLLIDE_ACCEPTANCE: f64 = 0.1; // Accept collision info when it is accurate to this fraction of delta t

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
    pub collide_normal: Option<(f32, Vector3<f64>)>, // Contains the time left in the collision frame and the normal of the collision
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
        let dt = if let Some((t, _)) = self.collide_normal {t as f64} else {delta_time};
        match self.updater {
            Updater::Fixed => (),
            Updater::Free => {
                self.vel += self.impulse / self.mass;
                self.pos += self.vel * dt;
                self.ang_vel += self.moi_inv() * (self.torque_impulse - self.ang_vel.cross(self.moi() * self.ang_vel) * delta_time);
                self.orientation += 0.5 * Quaternion::new(0.0, self.ang_vel.x, self.ang_vel.y, self.ang_vel.z) * self.orientation * dt;
                self.orientation = self.orientation.normalize();
                self.impulse = Vector3::zero();
                self.torque_impulse = Vector3::zero();
                self.collide_normal = None;
            },
            _ => unimplemented!()
        }
        
    }

    pub(crate) fn update_forceless(&mut self, delta_time: f64) {
        match self.updater {
            Updater::Fixed => (),
            Updater::Free => {
                self.pos += self.vel * delta_time;
                self.orientation += 0.5 * Quaternion::new(0.0, self.ang_vel.x, self.ang_vel.y, self.ang_vel.z) * self.orientation * delta_time;
                self.orientation = self.orientation.normalize();
            },
            _ => unimplemented!()
        }
    }

    pub(crate) fn detect_collision(&self, o: &RigidBody, delta_time: f64) -> Option<(f64, Vector3<f64>, Vector3<f64>)> {
        if let CollisionData::NoCollision = self.eval_collision(o, delta_time) {
            return None;
        }

        let mut high = 1.0;
        let mut low = 0.0; // Guaranteed to be no collision

        loop {
            if high - low < 0.001 {
                println!("Binary search got too tight.");
                // return None;
            }
            let mid = (high + low) / 2.0;
            match self.eval_collision(o, delta_time * mid) {
                CollisionData::NoCollision => { low = mid; },// Increase frac
                CollisionData::Collision(normal, center) => {
                    if high - low < COLLIDE_ACCEPTANCE {
                        return Some((mid * delta_time, normal, center)); // Accept collision
                    } else {
                        high = mid;// Decrease frac
                    }
                }, 
            }
        }
    }

    fn eval_collision(&self, o: &RigidBody, delta_t: f64) -> CollisionData {
        // Work in inertial frame, self-centric. Shift indicates the velocity of the other object. 
        let my_rb_pos = self.pos + self.vel * delta_t;
        let my_orientation = self.orientation + 0.5 * Quaternion::new(0.0, self.ang_vel.x, self.ang_vel.y, self.ang_vel.z) * self.orientation * delta_t;
        let my_orientation = my_orientation.normalize();
        let o_rb_pos = o.pos + o.vel * delta_t;
        let o_orientation = o.orientation + 0.5 * Quaternion::new(0.0, o.ang_vel.x, o.ang_vel.y, o.ang_vel.z) * o.orientation * delta_t;
        let o_orientation = o_orientation.normalize();
        let shift = delta_t as f64 * (o.vel - self.vel);
        
        for my_c in &self.colliders {
            for o_c in &o.colliders {
                // Confirm the objects are within collision distance
                let my_c_offset = my_orientation * my_c.offset();
                let o_c_offset = o_orientation * o_c.offset();
                let displacement = o_rb_pos - my_rb_pos - my_c_offset + o_c_offset;
                if displacement.magnitude() > my_c.radius() + o_c.radius() {
                    continue;
                }

                // Initialization
                let my_inv_orientation = my_orientation.invert();
                let o_inv_orientation = o_orientation.invert();
                let initial_axis = displacement.normalize();

                let my_pos = my_c.support(my_inv_orientation * initial_axis);
                let o_pos = o_c.support(o_inv_orientation * -initial_axis);
                let mut dir = -(my_orientation * my_pos - (o_orientation * o_pos + displacement));
                if shift.dot(initial_axis) > 0.0 {
                    dir -= shift;
                }
                let mut state = GJKState::new((-dir, my_pos, o_pos));
                loop {
                    dir = dir.normalize();
                    let my_pos = my_c.support(my_inv_orientation * dir);
                    let o_pos = o_c.support(o_inv_orientation * -dir);
                    let mut new_vec = my_orientation * my_pos - (o_orientation * o_pos + displacement);
                    if shift.dot(dir) > 0.0 {
                        new_vec += shift;
                    }

                    if new_vec.dot(dir) < 0.0 {
                        break;
                    }
                    state.push((new_vec, my_pos, o_pos));
                    if match state.contains_origin(&mut dir) {
                        Err(_) => break,
                        Ok(b) => b
                    } {
                        return state.get_collision_data(my_c_offset, my_c_offset - o_c_offset + o_rb_pos - my_rb_pos,
                            my_orientation, o_orientation);
                    }
                }
            }
        }
        CollisionData::NoCollision
    }
}