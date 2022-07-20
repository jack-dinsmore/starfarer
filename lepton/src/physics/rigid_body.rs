use cgmath::{Vector3, Matrix3, Quaternion, Matrix4, Zero, InnerSpace, SquareMatrix, Rotation};

use crate::shader::builtin;
use super::{Updater, Collider};

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
    pub(super) collider_offset: Vector3<f64>,
    pub(super) model_offset: Vector3<f32>,
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
            collider_offset: Vector3::zero(),
            model_offset: Vector3::new(0.0, 0.0, 0.0)
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
            collider_offset: Vector3::zero(),
            model_offset: Vector3::new(0.0, 0.0, 0.0)
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
    
    pub fn collide(mut self, collider: Collider, collider_offset: Vector3<f64>) -> Self {
        self.collider = collider;
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
            },
            _ => unimplemented!()
        }
        
    }

    pub(crate) fn detect_collision_bool(&self, o: &RigidBody) -> bool {
        // Work in axis-aligned frame
        if let Collider::None = self.collider {
            return false;
        }
        if let Collider::None = o.collider {
            return false;
        }

        let displacement = o.pos - self.pos - self.collider_offset + o.collider_offset;
        if displacement.magnitude() > self.collider.radius() + o.collider.radius() {
            return false;
        }
        let my_inv_orientation = self.orientation.invert();
        let o_inv_orientation = o.orientation.invert();

        let initial_axis = Vector3::new(1.0, 0.1, 0.06).normalize();
        let mut simplex = Vec::with_capacity(4);

        let mut dir = -(my_inv_orientation * self.collider.support(self.orientation * initial_axis)
            - (o_inv_orientation * o.collider.support(o.orientation * -initial_axis)) - displacement);
        simplex.push(-dir);
        let mut quit = false;
        while !quit {
            dir = dir.normalize();
            let new_vec = my_inv_orientation * self.collider.support(self.orientation * dir)
                - (o_inv_orientation * o.collider.support(o.orientation * -dir)) - displacement;
            let d = new_vec.dot(dir);
            if new_vec.dot(dir) < 0.0 {
                return false;
            }
            simplex.push(new_vec);
            if Collider::next_simplex(&mut simplex, &mut dir, &mut quit) {
                return true;
            }
        }
        false
    }
}