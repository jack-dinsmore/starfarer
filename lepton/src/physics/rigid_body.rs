use cgmath::{Vector3, Matrix3, Quaternion, Matrix4};

use crate::shader::builtin;
use super::{Updater, Collider};

pub struct RigidBody {
    pos: Vector3<f64>,
    vel: Vector3<f64>,
    mass: f64,

    orientation: Quaternion<f64>,
    ang_vel: Vector3<f64>, // Local frame
    moi: Matrix3<f64>,
    
    updater: Updater,
    
    radius: f64,
    collider: Collider,
    model_offset: Vector3<f32>,
}

impl RigidBody {
    pub fn new(pos: Vector3<f64>, vel: Vector3<f64>, orientation: Quaternion<f64>, ang_vel: Vector3<f64>,
        moi: Matrix3<f32>, model_offset: Vector3<f32>) -> Self {
        Self {
            pos,
            vel,
            mass: 0.0,
            orientation,
            ang_vel,
            moi: moi.cast().unwrap(),
            updater: Updater::Fixed,
            radius: 0.0,
            collider: Collider::None,
            model_offset
        }
    }

    pub fn by_pos(pos: Vector3<f64>) -> Self {
        Self {
            pos,
            vel: Vector3::new(0.0, 0.0, 0.0),
            mass: 0.0,
            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            ang_vel: Vector3::new(0.0, 0.0, 0.0),
            moi: Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
            updater: Updater::Fixed,
            radius: 0.0,
            collider: Collider::None,
            model_offset: Vector3::new(0.0, 0.0, 0.0)
        }
    }
}

impl RigidBody {
    pub(crate) fn push_constants(&self) -> builtin::ObjectPushConstants {
        let rotation = Matrix4::from(Matrix3::from(self.orientation.cast().unwrap()));
        builtin::ObjectPushConstants {
            model: Matrix4::from_translation(self.pos.cast().unwrap() + self.model_offset) * rotation,
            rotation: rotation,
        }
    }

    pub(crate) fn get_pos(&self) -> Vector3<f64> {
        self.pos
    }

    pub(crate) fn update(&mut self, delta_time: f64) {
        self.pos += self.vel * delta_time;
        self.orientation += 0.5 * self.orientation * Quaternion::new(0.0, self.ang_vel.x, self.ang_vel.y, self.ang_vel.z) * delta_time;
    }
}