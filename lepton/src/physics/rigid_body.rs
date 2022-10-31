use cgmath::{Vector3, Matrix3, Quaternion, Matrix4, Matrix, Zero, InnerSpace, SquareMatrix};

use crate::shader::builtin;
use super::{Updater, Collider, collider::CollisionData};

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
    pub collide_data: Option<(f32, Vector3<f64>, Vector3<f64>)>, // delta_t so far, pos, normal
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
            collide_data: None,
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
            collide_data: None,
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

    pub fn orbit(mut self, center: Vector3<f64>, normal: Vector3<f64>) -> Self {
        let perigee = self.pos - center;
        let right = perigee.cross(normal) / (normal.magnitude());
        self.updater = Updater::Orbit{center, perigee, right, period: 5.0};
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
        let dt = if let Some((t, _, _)) = self.collide_data {t as f64} else {delta_time};
        match self.updater {
            Updater::Fixed => (),
            Updater::Free => {
                if let Some((_, r, n)) = self.collide_data {
                    let impulse_into_ground = self.impulse - r.cross(self.moi_inv() * self.torque_impulse);
                    self.impulse += n * n.dot(impulse_into_ground);
                }
                self.vel += self.impulse / self.mass;
                self.pos += self.vel * dt;
                self.ang_vel += self.moi_inv() * (self.torque_impulse - self.ang_vel.cross(self.moi() * self.ang_vel) * delta_time);
                self.orientation += 0.5 * Quaternion::new(0.0, self.ang_vel.x, self.ang_vel.y, self.ang_vel.z) * self.orientation * dt;
                self.orientation = self.orientation.normalize();
                self.impulse = Vector3::zero();
                self.torque_impulse = Vector3::zero();
                self.collide_data = None;
            },
            Updater::Orbit{center, perigee, right, period} => {
                let orbit_pos = (self.pos - center).normalize();
                let x = perigee.dot(orbit_pos);
                let y = right.dot(orbit_pos);
                let angle = f64::atan2(y, x);
                let new_angle = angle + 2.0 * std::f64::consts::PI * delta_time / period;
                self.pos = center + perigee * new_angle.cos() + right * new_angle.sin();
            }
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

                let collision_data = if let Collider::Radial{..} = my_c {
                    if let Collider::Radial{..} = o_c {
                        // No planet-planet collisions
                        continue;
                    }
                    // Planet gjk
                    Collider::planet_collide(my_c, o_c, my_rb_pos, o_rb_pos, my_orientation, o_orientation, shift)
                } else if let Collider::Radial{..} = o_c {
                    // Planet gjk
                    Collider::planet_collide(o_c, my_c, o_rb_pos, my_rb_pos, o_orientation, my_orientation, -shift)
                } else {
                    // Normal gjk
                    Collider::gjk_collide(my_c, o_c, my_rb_pos, o_rb_pos, my_orientation, o_orientation, shift)
                };

                if let Some(data) = collision_data {
                    return data;
                }
            }
        }
        CollisionData::NoCollision
    }
}