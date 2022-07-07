mod object;

pub use object::*;
use cgmath::{Vector3};

pub struct Physics {

}

enum Updater {
    Fixed,
    Free,
    Line,
    Circle,
    Orbit,
}

enum Collider {
    Box,
    Sphere,
    Plane,
    BoundedPlane,
}

pub struct RigidBody<'a> {
    object: &'a Object,
    
    vel: Vector3<f64>,
    force: Vector3<f64>,
    mass: f64,

    ang_vel: Vector3<f64>, // Local frame
    torque: Vector3<f64>,
    
    updater: Updater,
    
    radius: f64,
    collider: Collider,
}

impl Physics {
    pub fn new() -> Physics {
        Physics {}
    }
}