// GKJ algorithm: http://realtimecollisiondetection.net/pubs/SIGGRAPH04_Ericson_GJK_notes.pdf

use cgmath::{Vector3, InnerSpace};
use anyhow::{Result, anyhow};
use rustc_hash::FxHashMap;

const EPSILON: f64 = 1e-10;
const MAX_ITERATIONS: u32 = 20;
type Vertex = (Vector3<f64>, usize, usize);

pub enum Collider {
    None,
    Cube{length: f64},
}

impl Collider {
    pub fn cube(length: f64) -> Self {
        Collider::Cube{ length }
    }
}

impl Collider {
    pub(super) fn support(&self, dir: Vector3<f64>) -> (usize, Vector3<f64>) {
        let id = match self {
            Collider::Cube{..} => {
                if dir.x > 0.0 {
                    if dir.y > 0.0 {
                        if dir.z > 0.0 {
                            0
                        } else {
                            1
                        }
                    } else {
                        if dir.z > 0.0 {
                            2
                        } else {
                            3
                        }
                    }
                } else {
                    if dir.y > 0.0 {
                        if dir.z > 0.0 {
                            4
                        } else {
                            5
                        }
                    } else {
                        if dir.z > 0.0 {
                            6
                        } else {
                            7
                        }
                    }
                }
            },
            Collider::None => panic!("Called support on None collider"),
        };
        (id, self.id_to_vertex(id))
    }

    pub(super) fn id_to_vertex(&self, id: usize) -> Vector3<f64> {
        match self {
            Collider::None => panic!("Called id select on None collider"),
            Collider::Cube{length} => {
                match id {
                    0 => Vector3::new(*length, *length, *length),
                    1 => Vector3::new(*length, *length, -*length),
                    2 => Vector3::new(*length, -*length, *length),
                    3 => Vector3::new(*length, -*length, -*length),
                    4 => Vector3::new(-*length, *length, *length),
                    5 => Vector3::new(-*length, *length, -*length),
                    6 => Vector3::new(-*length, -*length, *length),
                    7 => Vector3::new(-*length, -*length, -*length),
                    _ => unreachable!(),
                }
            }
        }
    }

    pub(super) fn radius(&self) -> f64 {
        match self {
            Collider::Cube{length} =>  *length * 2.0f64.sqrt(),
            _ => unreachable!()
        }
    }
}

pub(super) enum CollisionType {
    EdgeEdge((usize, usize), (usize, usize)),
    FaceVertex((usize, usize, usize), (usize,)),
    VertexFace((usize,), (usize, usize, usize)),
    FaceFace((usize, usize, usize), (usize, usize, usize)),
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

    pub fn get_collision_type(&mut self) -> Result<CollisionType> {
        Ok(if let Simplex::Tetrahedron(a, b, c, d) = self.simplex {
            let mut my_vertices = FxHashMap::default();
            let mut o_vertices = FxHashMap::default();
            let vertex_array = [a, b, c, d];
            for v in &vertex_array {
                if my_vertices.contains_key(&v.1) {
                    *my_vertices.get_mut(&v.1).unwrap() += 1;
                } else {
                    my_vertices.insert(v.1, 1);
                }
                if o_vertices.contains_key(&v.2) {
                    *o_vertices.get_mut(&v.2).unwrap() += 1;
                } else {
                    o_vertices.insert(v.2, 1);
                }
            }

            let mut my_three = None;
            let mut my_two = None;
            let mut o_three = None;
            let mut o_two = None;
            for (k, v) in &my_vertices {
                if *v == 3 {
                    my_three = Some(k);
                }
                if *v == 2 {
                    my_two = Some(k);
                }
            }
            for (k, v) in &o_vertices {
                if *v == 3 {
                    o_three = Some(k);
                }
                if *v == 2 {
                    o_two = Some(k);
                }
            }

            if let Some(v) = my_three { // My vertex
                CollisionType::VertexFace(
                    (*v,), 
                    (*o_vertices.keys().nth(0).ok_or(anyhow!("Face case didn't find three vertices; {:?}, {:?}", my_vertices, o_vertices))?,
                    *o_vertices.keys().nth(1).ok_or(anyhow!("Face case didn't find three vertices; {:?}, {:?}", my_vertices, o_vertices))?,
                    *o_vertices.keys().nth(2).ok_or(anyhow!("Face case didn't find three vertices; {:?}, {:?}", my_vertices, o_vertices))?))

            } else if let Some(v) = o_three { // Other vertex
                CollisionType::FaceVertex(
                    (*my_vertices.keys().nth(0).ok_or(anyhow!("Face case didn't find three vertices; {:?}, {:?}", my_vertices, o_vertices))?,
                    *my_vertices.keys().nth(1).ok_or(anyhow!("Face case didn't find three vertices; {:?}, {:?}", my_vertices, o_vertices))?,
                    *my_vertices.keys().nth(2).ok_or(anyhow!("Face case didn't find three vertices; {:?}, {:?}", my_vertices, o_vertices))?),
                    (*v,))
            } else if let Some(my) = my_two { // My edge
                match o_two {
                    Some(o) => {
                        let mut my_verts = (None, None);
                        let mut o_verts = (None, None);
                        for v in &vertex_array {
                            if v.1 == *my {
                                if o_verts.0.is_some() {
                                    o_verts.1 = Some(v.2)
                                } else {
                                    o_verts.0 = Some(v.2)
                                }
                            }
                            if v.2 == *o {
                                if my_verts.0.is_some() {
                                    my_verts.1 = Some(v.1)
                                } else {
                                    my_verts.0 = Some(v.1)
                                }
                            }
                        }
                        CollisionType::EdgeEdge(
                            (my_verts.0.unwrap(), 
                            my_verts.1.unwrap()), 
                            (o_verts.0.unwrap(), 
                            o_verts.1.unwrap())
                        )
                    },
                    None => {
                        let mut my_others = (None, None);
                        for v in &vertex_array {
                            if v.1 != *my {
                                if my_others.0.is_some() {
                                    my_others.1 = Some(v.1);
                                } else {
                                    my_others.0 = Some(v.1);
                                }
                            }
                        }
                        CollisionType::FaceFace(
                            (*my, my_others.0.unwrap(), my_others.1.unwrap()),
                            (vertex_array[0].2, vertex_array[1].2, vertex_array[2].2)
                        )
                    },
                }
            } else if let Some(o) = o_two { // O edge
                // both edges covered
                let mut o_others = (None, None);
                for v in &vertex_array {
                    if v.2 != *o {
                        if o_others.0.is_some() {
                            o_others.1 = Some(v.2);
                        } else {
                            o_others.0 = Some(v.2);
                        }
                    }
                }
                CollisionType::FaceFace(
                    (vertex_array[0].1, vertex_array[1].1, vertex_array[2].1),
                    (*o, o_others.0.unwrap(), o_others.1.unwrap()),
                )
            } else {
                CollisionType::FaceFace(
                    (vertex_array[0].1, vertex_array[1].1, vertex_array[2].1),
                    (vertex_array[0].2, vertex_array[1].2, vertex_array[2].2)
                )
            }
        } else {
            unreachable!();
        })
    }
}

enum Simplex {
    Point(Vertex),
    Line(Vertex, Vertex),
    Triangle(Vertex, Vertex, Vertex),
    Tetrahedron(Vertex, Vertex, Vertex, Vertex),
}

impl Simplex {
    pub fn push(&mut self, v: Vertex) {
        *self = match self {
            Simplex::Point(i) => Simplex::Line(*i, v),
            Simplex::Line(i, j) => Simplex::Triangle(*i, *j, v),
            Simplex::Triangle(i, j, k) => Simplex::Tetrahedron(*i, *j, *k, v),
            Simplex::Tetrahedron(..) => unreachable!(),
        };
    }

    /// Get the closest point to the origin and restrict to the simplex that contains it. None is
    /// returned if the closest point containing the origin is inside the simplex.
    pub fn downgrade(&mut self) -> Option<Vector3<f64>> {
        match self {
            Self::Tetrahedron(a, b, c, d) => {
                // Vertices
                let (v01, v02, v03) = (a.0.dot(b.0 - a.0) >= 0.0, a.0.dot(c.0 - a.0) >= 0.0, a.0.dot(d.0 - a.0) >= 0.0);
                let (v10, v12, v13) = (b.0.dot(a.0 - b.0) >= 0.0, b.0.dot(c.0 - b.0) >= 0.0, b.0.dot(d.0 - b.0) >= 0.0);
                let (v20, v21, v23) = (c.0.dot(a.0 - c.0) >= 0.0, c.0.dot(b.0 - c.0) >= 0.0, c.0.dot(d.0 - c.0) >= 0.0);
                let (v30, v31, v32) = (d.0.dot(a.0 - d.0) >= 0.0, d.0.dot(b.0 - d.0) >= 0.0, d.0.dot(c.0 - d.0) >= 0.0);
                if v01 && v02 && v03 {
                    let out = a.0;
                    *self = Self::Point(*a);
                    Some(out)
                } else if v10 && v12 && v13 {
                    let out = b.0;
                    *self = Self::Point(*b);
                    Some(out)
                } else if v20 && v21 && v23 {
                    let out = c.0;
                    *self = Self::Point(*c);
                    Some(out)
                } else if v30 && v31 && v32 {
                    let out = d.0;
                    *self = Self::Point(*d);
                    Some(out)
                } else {
                    // Lines
                    let n012 = (b.0 - a.0).cross(c.0 - a.0);
                    let n013 = (b.0 - a.0).cross(d.0 - a.0);
                    let n023 = (c.0 - a.0).cross(d.0 - a.0);
                    
                    let n102 = (a.0 - b.0).cross(c.0 - b.0);
                    let n103 = (a.0 - b.0).cross(d.0 - b.0);
                    let n123 = (c.0 - b.0).cross(d.0 - b.0);

                    let n203 = (a.0 - c.0).cross(d.0 - c.0);
                    let n213 = (b.0 - c.0).cross(d.0 - c.0);
                    if !v01 && !v10 && a.0.dot((b.0 - a.0).cross(n012)) <= 0.0 && a.0.dot((b.0 - a.0).cross(n013)) <= 0.0 {
                        let line = a.0 - b.0;
                        let out = a.0 - line * line.dot(a.0) / line.magnitude2();
                        *self = Self::Line(*a, *b);
                        Some(out)
                    } else if !v02 && !v20 && a.0.dot((c.0 - a.0).cross(-n012)) <= 0.0 && a.0.dot((c.0 - a.0).cross(n023)) <= 0.0 {
                        let line = a.0 - c.0;
                        let out = a.0 - line * line.dot(a.0) / line.magnitude2();
                        *self = Self::Line(*a, *c);
                        Some(out)
                    } else if !v03 && !v30 && a.0.dot((d.0 - a.0).cross(-n013)) <= 0.0 && a.0.dot((d.0 - a.0).cross(-n023)) <= 0.0 {
                        let line = a.0 - d.0;
                        let out = a.0 - line * line.dot(a.0) / line.magnitude2();
                        *self = Self::Line(*a, *d);
                        Some(out)
                    } else if !v12 && !v21 && b.0.dot((c.0 - b.0).cross(-n102)) <= 0.0 && b.0.dot((c.0 - b.0).cross(n123)) <= 0.0 {
                        let line = b.0 - c.0;
                        let out = b.0 - line * line.dot(b.0) / line.magnitude2();
                        *self = Self::Line(*b, *c);
                        Some(out)
                    } else if !v13 && !v31 && b.0.dot((d.0 - b.0).cross(-n103)) <= 0.0 && b.0.dot((d.0 - b.0).cross(-n123)) <= 0.0 {
                        let line = b.0 - d.0;
                        let out = b.0 - line * line.dot(b.0) / line.magnitude2();
                        *self = Self::Line(*b, *d);
                        Some(out)
                    } else if !v23 && !v32 && c.0.dot((d.0 - c.0).cross(-n203)) <= 0.0 && c.0.dot((d.0 - c.0).cross(-n213)) <= 0.0 {
                        let line = c.0 - d.0;
                        let out = c.0 - line * line.dot(c.0) / line.magnitude2();
                        *self = Self::Line(*c, *d);
                        Some(out)
                    } else {
                        // Faces
                        if n123.dot(b.0) * n123.dot(b.0 - a.0) < 0.0 {
                            let out = n123 * b.0.dot(n123) / n123.magnitude2();
                            *self = Self::Triangle(*b, *c, *d);
                            Some(out)
                        } else if n023.dot(a.0) * n023.dot(a.0 - b.0) < 0.0 {
                            let out = n023 * a.0.dot(n023) / n023.magnitude2();
                            *self = Self::Triangle(*a, *c, *d);
                            Some(out)
                        } else if n013.dot(a.0) * n013.dot(a.0 - c.0) < 0.0 {
                            let out = n013 * a.0.dot(n013) / n013.magnitude2();
                            *self = Self::Triangle(*a, *b, *d);
                            Some(out)
                        } else if n012.dot(a.0) * n012.dot(a.0 - d.0) < 0.0 {
                            let out = n012 * a.0.dot(n012) / n012.magnitude2();
                            *self = Self::Triangle(*a, *b, *c);
                            Some(out)
                        } else {
                            // Volume
                            None
                        }
                    }
                }
            },
            Self::Triangle(a, b, c) => {
                // Vertices
                let (v01, v02) = (a.0.dot(b.0 - a.0) >= 0.0, a.0.dot(c.0 - a.0) >= 0.0);
                let (v10, v12) = (b.0.dot(a.0 - b.0) >= 0.0, b.0.dot(c.0 - b.0) >= 0.0);
                let (v20, v21) = (c.0.dot(a.0 - c.0) >= 0.0, c.0.dot(b.0 - c.0) >= 0.0);
                if v01 && v02 {
                    let out = a.0;
                    *self = Self::Point(*a);
                    Some(out)
                } else if v10 && v12 {
                    let out = b.0;
                    *self = Self::Point(*b);
                    Some(out)
                } else if v20 && v21 {
                    let out = c.0;
                    *self = Self::Point(*c);
                    Some(out)
                } else {
                    // Lines
                    let n012 = (b.0 - a.0).cross(c.0 - a.0);
                    if !v01 && !v10 && a.0.dot((b.0 - a.0).cross(n012)) <= 0.0 {
                        let line = a.0 - b.0;
                        let out = a.0 - line * line.dot(a.0) / line.magnitude2();
                        *self = Self::Line(*a, *b);
                        Some(out)
                    } else if !v02 && !v20 && a.0.dot((c.0 - a.0).cross(-n012)) <= 0.0 {
                        let line = a.0 - c.0;
                        let out = a.0 - line * line.dot(a.0) / line.magnitude2();
                        *self = Self::Line(*a, *c);
                        Some(out)
                    } else if !v12 && !v21 && b.0.dot((c.0 - b.0).cross(n012)) <= 0.0 {
                        let line = b.0 - c.0;
                        let out = b.0 - line * line.dot(b.0) / line.magnitude2();
                        *self = Self::Line(*b, *c);
                        Some(out)
                    } else {
                        // Faces
                        let out = n012 * a.0.dot(n012) / n012.magnitude2();
                        Some(out)
                    }
                }
            },
            Self::Line(a, b) => {
                let line = b.0 - a.0;
                let a_sign = line.dot(a.0);
                let b_sign = line.dot(b.0);
                if (a_sign > 0.0) ^ (b_sign > 0.0) {
                    // Line
                    Some(a.0 - line * a_sign / line.magnitude2())
                } else {
                    // Points
                    if a_sign > 0.0 {
                        let out = a.0;
                        *self = Self::Point(*a);
                        Some(out)
                    } else {
                        let out = b.0;
                        *self = Self::Point(*b);
                        Some(out)
                    }
                }
            }
            Self::Point(a) => {
                Some(a.0)
            }
        }
    }
}