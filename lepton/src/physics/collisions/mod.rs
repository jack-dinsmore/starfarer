use cgmath::{Vector3, InnerSpace, Zero};
use super::RigidBody;

const EPSILON: f64 = 1e-10;
pub enum Collider {
    None,
    Sphere{radius: f64},
    Cube{length: f64},
}

impl Collider {
    pub fn sphere(radius: f64) -> Self {
        Collider::Sphere{ radius }
    }
    pub fn cube(length: f64) -> Self {
        Collider::Cube{ length }
    }
}

impl Collider {
    pub(super) fn support(&self, dir: Vector3<f64>) -> Vector3<f64> {
        match self {
            Collider::Sphere{radius} => dir * *radius,
            Collider::Cube{length} => {
                if dir.x.abs() > dir.y.abs() && dir.x.abs() > dir.z.abs() {
                    if dir.x > 0.0 {
                        Vector3::new(*length, 0.0, 0.0)
                    } else {
                        Vector3::new(-*length, 0.0, 0.0)
                    }
                } else if dir.y.abs() > dir.x.abs() && dir.y.abs() > dir.z.abs() {
                    if dir.x > 0.0 {
                        Vector3::new(0.0, *length, 0.0)
                    } else {
                        Vector3::new(0.0, -*length, 0.0)
                    }
                } else {
                    if dir.z > 0.0 {
                        Vector3::new(0.0, 0.0, *length)
                    } else {
                        Vector3::new(0.0, 0.0, -*length)
                    }
                }
            },
            _ => unreachable!()
        }
    }

    pub(super) fn radius(&self) -> f64 {
        match self {
            Collider::Sphere{radius} =>  *radius,
            Collider::Cube{length} =>  *length * 2.0f64.sqrt(),
            _ => unreachable!()
        }
    }

    pub(super) fn next_simplex(simplex: &mut Vec<Vector3<f64>>, dir: &mut Vector3<f64>, quit: &mut bool) -> bool {
        match simplex.len() {
            2 => {
                *dir = simplex[0].cross(simplex[1]).cross(simplex[1] - simplex[0]);
                if dir.dot(simplex[0]) > 0.0 {
                    *dir *= -1.0;
                }
                false
            },
            3 => {
                *dir = (simplex[1] - simplex[0]).cross(simplex[2] - simplex[0]);
                if dir.dot(simplex[0]) > 0.0 {
                    *dir *= -1.0;
                }
                false
            },
            4 => {
                // Contains origin?
                let mut min_dist = f64::MAX;
                let mut min_drop_index = 0;
                let mut min_normal = Vector3::zero();
                let mut contains_origin = true;
                for i in 0..4 {
                    let normal = (simplex[(i + 2) % 4] - simplex[(i + 1) % 4]).cross(
                        simplex[(i + 3) % 4] - simplex[(i + 1) % 4]
                    );
                    let dist = normal.dot(simplex[(i + 1) % 4]);
                    let remaining_dist = normal.dot(simplex[(i) % 4]);
                    if dist.abs() < EPSILON {
                        *quit = true;
                    } else if !((dist > 0.0) ^ (remaining_dist > 0.0)) {
                        contains_origin = false;
                    }
                    if dist.abs() < min_dist {
                        min_dist = dist.abs() / normal.magnitude();
                        min_drop_index = i;
                        min_normal = normal;
                    }
                }
                if contains_origin {
                    return true;
                }
                *dir = min_normal;
                simplex.remove(min_drop_index);
                false
            },
            _ => unreachable!()
        }
    }
}