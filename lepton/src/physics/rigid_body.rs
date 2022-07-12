use cgmath::{Vector3, Matrix3, Quaternion, Matrix4};

use crate::shader::builtin;
use super::{Updater, Collider};

pub struct RigidBody {
    pos: Vector3<f64>,
    vel: Vector3<f64>,
    mass: f64,

    orientation: Quaternion<f64>,
    ang_vel: Vector3<f64>, // Local frame
    
    updater: Updater,
    
    radius: f64,
    collider: Collider,
}

impl RigidBody {
    pub fn new(pos: Vector3<f64>, vel: Vector3<f64>, orientation: Quaternion<f64>, ang_vel: Vector3<f64>) -> Self {
        Self {
            pos,
            vel,
            mass: 0.0,
            orientation,
            ang_vel,
            updater: Updater::Fixed,
            radius: 0.0,
            collider: Collider::None,
        }
    }

    pub fn still(pos: Vector3<f64>, orientation: Quaternion<f64>) -> Self {
        Self {
            pos,
            vel: Vector3::new(0.0, 0.0, 0.0),
            mass: 0.0,
            orientation,
            ang_vel: Vector3::new(0.0, 0.0, 0.0),
            updater: Updater::Fixed,
            radius: 0.0,
            collider: Collider::None,
        }
    }
}

impl RigidBody {
    pub(crate) fn push_constants(&self) -> builtin::ObjectPushConstants {
        let rotation = Matrix4::from(Matrix3::from(self.orientation.cast().unwrap()));
        builtin::ObjectPushConstants {
            model: Matrix4::from_translation(self.pos.cast().unwrap()) * rotation,
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