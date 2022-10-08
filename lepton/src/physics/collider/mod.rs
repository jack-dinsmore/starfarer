// GKJ algorithm: http://realtimecollisiondetection.net/pubs/SIGGRAPH04_Ericson_GJK_notes.pdf
mod primitives;

use primitives::*;
pub use primitives::CollisionData;

use cgmath::{Vector3, Quaternion, InnerSpace, Zero, Rotation};
use anyhow::{Result, anyhow};

const EPSILON: f64 = 1e-10;
const MAX_ITERATIONS: u32 = 20;

pub enum Collider {
    Cube{length: f64},
    Radial{func: Box<dyn 'static + Send + Sync + Fn(Vector3<f64>) -> f64>, length: f64 },
    Polyhedron{vertices: Vec<Vector3<f64>>, length: f64, offset: Vector3<f64>}
}

impl Collider {
    /// Temporary cube collider initialization function
    pub fn cube(length: f64) -> Self {
        Collider::Cube{ length }
    }
    
    /// A planetary collider function. Pass in the value function and the length
    pub fn planet(func: Box<dyn 'static + Send + Sync + Fn(Vector3<f64>) -> f64>, length: f64) -> Self {
        Collider::Radial { func, length }
    }

    /// Polyhedral function, mostly for ships
    pub fn polyhedron(vertices: Vec<Vector3<f64>>) -> Self {
        let offset = vertices.iter().map(|v| v).sum::<Vector3<f64>>() / vertices.len() as f64;
        let vertices = vertices.into_iter().map(|v| v - offset).collect::<Vec<Vector3<f64>>>();
        let length = vertices.iter().map(|v| v.magnitude2()).max_by(|a, b| a.total_cmp(b)).unwrap().sqrt();
        Collider::Polyhedron { vertices, length, offset }
    }
}

impl Collider {
    pub(super) fn support(&self, dir: Vector3<f64>) -> Vector3<f64> {
        match self {
            Collider::Cube{length} => {
                if dir.x > 0.0 {
                    if dir.y > 0.0 {
                        if dir.z > 0.0 {
                            Vector3::new(*length, *length, *length)
                        } else {
                            Vector3::new(*length, *length, -*length)
                        }
                    } else {
                        if dir.z > 0.0 {
                            Vector3::new(*length, -*length, *length)
                        } else {
                            Vector3::new(*length, -*length, -*length)
                        }
                    }
                } else {
                    if dir.y > 0.0 {
                        if dir.z > 0.0 {
                            Vector3::new(-*length, *length, *length)
                        } else {
                            Vector3::new(-*length, *length, -*length)
                        }
                    } else {
                        if dir.z > 0.0 {
                            Vector3::new(-*length, -*length, *length)
                        } else {
                            Vector3::new(-*length, -*length, -*length)
                        }
                    }
                }
            },
            Collider::Polyhedron { vertices, .. } => {
                //// Later, implement a binary tree?
                let mut max_dot = 0.0;
                let mut max_v = None;
                for v in vertices {
                    let dot = v.dot(dir);
                    if dot > max_dot || max_v.is_none() {
                        max_dot = dot;
                        max_v = Some(v);
                    }
                }
                *max_v.unwrap()
            },
            _ => panic!("Collider not provided with a support function")
        }
    }

    pub(super) fn radius(&self) -> f64 {
        match self {
            Collider::Cube{length} =>  *length * 2.0f64.sqrt(),
            Collider::Radial{length, ..} =>  *length,
            Collider::Polyhedron{length, ..} =>  *length,
        }
    }

    pub(super) fn offset(&self) -> Vector3<f64> {
        match self {
            Collider::Cube{..} => Vector3::zero(),
            Collider::Radial{..} => Vector3::zero(),
            Collider::Polyhedron { offset, ..} => *offset,
        }
    }

    pub(super) fn gjk_collide(my_c: &Collider, o_c: &Collider,
        my_rb_pos: Vector3<f64>, o_rb_pos: Vector3<f64>,
        my_orientation: Quaternion<f64>, o_orientation: Quaternion<f64>,
        shift: Vector3<f64>) -> Option<CollisionData> {

        // Confirm the objects are within collision distance
        let my_c_offset = my_orientation * my_c.offset();
        let o_c_offset = o_orientation * o_c.offset();
        let displacement = o_rb_pos - my_rb_pos - my_c_offset + o_c_offset;
        if displacement.magnitude() > my_c.radius() + o_c.radius() {
            return None;
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
                return Some(state.get_collision_data(my_c_offset, my_c_offset - o_c_offset + o_rb_pos - my_rb_pos,
                    my_orientation, o_orientation));
            }
        }
        None
    }

    pub(super) fn planet_collide(planet: &Collider, c: &Collider,
        planet_pos: Vector3<f64>, c_pos: Vector3<f64>,
        planet_orientation: Quaternion<f64>, c_orientation: Quaternion<f64>,
        shift: Vector3<f64>) -> Option<CollisionData> {
            // Confirm the objects are within collision distance
        let planet_offset = planet_orientation * planet.offset();
        let c_offset = c_orientation * c.offset();
        let displacement = c_pos - planet_pos - planet_offset + c_offset - shift;
        if displacement.magnitude() > planet.radius() + c.radius() {
            return None;
        }

        let lowermost_point = c_orientation * c.support(-c_orientation.invert() * displacement.normalize()) - displacement;
        if let Self::Radial{ func, ..} = planet {
            let value = func(lowermost_point);
            if value < 0.0 {
                // Collision
                let lowermost_mag = lowermost_point.magnitude();
                let normal = lowermost_point / lowermost_mag;
                let point = lowermost_point * (1.0 + value / lowermost_mag);
                return Some(CollisionData::Collision(point, normal));
            }
        } else {
            unreachable!();
        }
        None
    }
}

pub(super) enum CollisionType {
    EdgeEdge((Vector3<f64>, Vector3<f64>), (Vector3<f64>, Vector3<f64>)),
    FaceVertex((Vector3<f64>, Vector3<f64>, Vector3<f64>), (Vector3<f64>,)),
    VertexFace((Vector3<f64>,), (Vector3<f64>, Vector3<f64>, Vector3<f64>)),
    Other,
}

pub(super) struct GJKState {
    simplex: Simplex,
    num_iter: u32,
}

impl GJKState {
    pub fn new(point: Vertex) -> Self {
        Self {
            simplex: Simplex::Point(point),
            num_iter: 0,
        }
    }
    
    pub fn push(&mut self, point: Vertex) {
        self.simplex.push(point);
    }

    /// Returns true if the simplex contains the origin. Otherwise, returns the direction in which
    /// to search for the next point. That's the negative of the position of the closest point in
    /// the simplex to the origin. If the state decides the operation is taking too long, err is 
    /// returned.
    pub fn contains_origin(&mut self, dir: &mut Vector3<f64>) -> Result<bool> {
        let p = match self.simplex.downgrade() {
            Some(p) => p,
            None => return Ok(true) // Intersection detected.
        };
        *dir = -p;
        self.num_iter += 1;
        if self.num_iter > MAX_ITERATIONS {
            return Err(anyhow!("Too many iterations"));
        }
        Ok(false)
    }

    pub fn get_collision_data(&mut self, my_offset: Vector3<f64>, o_offset: Vector3<f64>,
        my_orientation: Quaternion<f64>, o_orientation: Quaternion<f64>) -> CollisionData {

        if let Simplex::Tetrahedron(a, b, c, d) = self.simplex {
            let my_simplex = VertexCounter::reduce(
                my_orientation * a.1 + my_offset,
                my_orientation * b.1 + my_offset,
                my_orientation * c.1 + my_offset,
                my_orientation * d.1 + my_offset);
            let o_simplex = VertexCounter::reduce(
                o_orientation * a.2 + o_offset,
                o_orientation * b.2 + o_offset,
                o_orientation * c.2 + o_offset,
                o_orientation * d.2 + o_offset);

            VertexCounter::collide_data(my_simplex, o_simplex)
        } else {
            unreachable!()
        }
    }
}