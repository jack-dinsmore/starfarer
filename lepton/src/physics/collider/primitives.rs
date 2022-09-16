use cgmath::{Vector3, InnerSpace};

pub type Vertex = (Vector3<f64>, Vector3<f64>, Vector3<f64>); // ?, my vertex, o vertex


pub enum Simplex {
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

#[derive(Debug)]
pub enum VertexCounter {
    Point(Vector3<f64>),
    Line(Vector3<f64>, Vector3<f64>),
    Triangle(Vector3<f64>, Vector3<f64>, Vector3<f64>),
    Tetrahedron(Vector3<f64>, Vector3<f64>, Vector3<f64>, Vector3<f64>),
}

impl VertexCounter {
    pub fn reduce(a: Vector3<f64>, b: Vector3<f64>, c: Vector3<f64>, d: Vector3<f64>) -> Self {
        let mut vertices_out = Vec::new();
        let vertices_in = [a, b, c, d];
        for v in vertices_in {
            if !vertices_out.contains(&v) {
                vertices_out.push(v);
            }
        }
        match vertices_out.len() {
            1 => Self::Point(vertices_out[0]),
            2 => Self::Line(vertices_out[0], vertices_out[1]),
            3 => Self::Triangle(vertices_out[0], vertices_out[1], vertices_out[2]),
            4 => Self::Tetrahedron(vertices_out[0], vertices_out[1], vertices_out[2], vertices_out[3]),
            _ => unreachable!(),
        }
    }

    pub fn collide_data(one: Self, two: Self) -> CollisionData {
        match one {
            Self::Point(_) => match two {
                Self::Point(_) => unreachable!(),
                Self::Line(..) => unreachable!(),
                Self::Triangle(..) => unreachable!(),
                Self::Tetrahedron(..) => Self::point_tet(one, two),
            },
            Self::Line(..) => match two {
                Self::Point(_) => unreachable!(),
                Self::Line(..) => Self::edge_edge(one, two),
                Self::Triangle(..) => Self::edge_face(one, two),
                Self::Tetrahedron(..) => Self::edge_tet(one, two),
            },
            Self::Triangle(..) => match two {
                Self::Point(_) => unreachable!(),
                Self::Line(..) => Self::edge_face(two, one),
                Self::Triangle(..) => Self::face_face(one, two),
                Self::Tetrahedron(..) => Self::face_tet(one, two),
            },
            Self::Tetrahedron(..) => match two {
                Self::Point(_) => Self::point_tet(two, one),
                Self::Line(..) => Self::edge_tet(two, one),
                Self::Triangle(..) => Self::face_tet(two, one),
                Self::Tetrahedron(..) => Self::tet_tet(two, one),
            },
        }
    }

    fn point_tet(point: Self, tet: Self) -> CollisionData {
        if let Self::Point(a1) = point {
            if let Self::Tetrahedron(a2, b2, c2, d2) = tet {
                let normals_pos = [
                    ((a2 - c2).cross(b2 - c2), c2),
                    ((a2 - d2).cross(b2 - d2), d2),
                    ((a2 - d2).cross(c2 - d2), d2),
                    ((b2 - d2).cross(c2 - d2), d2),
                ];
                let mut min_normal = None;
                let mut min_dot = f64::INFINITY;
                for (n, p) in normals_pos {
                    let dot = (a1 - p).dot(n).abs();
                    if dot < min_dot {
                        min_normal = Some(n);
                        min_dot = dot;
                    }
                }
                let normal = min_normal.unwrap();
                return CollisionData::Collision(normal.normalize(), a1);
            }
        }
        unreachable!()
    }

    fn edge_edge(edge1: Self, edge2: Self) -> CollisionData {
        if let Self::Line(a1, b1) = edge1 {
            if let Self::Line(a2, b2) = edge2 {
                let normal = (b1 - a1).cross(b2-a2);
                let point1 = - normal.dot(a1) / normal.dot(b1 - a1) * (b1 - a1) + a1;
                let point2 = - normal.dot(a2) / normal.dot(b2 - a2) * (b2 - a2) + a2;
                return CollisionData::Collision(normal.normalize(), (point1 + point2) / 2.0);
            }
        }
        unreachable!()
    }

    fn edge_face(edge: Self, face: Self) -> CollisionData {
        if let Self::Line(a1, b1) = edge {
            if let Self::Triangle(a2, b2, c2) = face {
                let normal = (a2 - c2).cross(b2 - c2);
                let dot_a = normal.dot(a1 - c2).abs();
                let dot_b = normal.dot(b1 - c2).abs();
                if dot_a < dot_b {
                    return CollisionData::Collision(normal.normalize(), a1);
                } else {
                    return CollisionData::Collision(normal.normalize(), b1);
                }
            }
        }
        unreachable!()
    }

    fn edge_tet(edge: Self, tet: Self) -> CollisionData {
        if let Self::Line(a1, b1) = edge {
            if let Self::Tetrahedron(a2, b2, c2, d2) = tet {
                let normals_pos = [
                    ((a2 - c2).cross(b2 - c2), c2),
                    ((a2 - d2).cross(b2 - d2), d2),
                    ((a2 - d2).cross(c2 - d2), d2),
                    ((b2 - d2).cross(c2 - d2), d2),
                ];
                let mut min_normal = None;
                let mut min_point = None;
                let mut min_dot = f64::INFINITY;
                for (n, p) in normals_pos {
                    let dot_a = (a1 - p).dot(n).abs();
                    let dot_b = (b1 - p).dot(n).abs();
                    if dot_a < min_dot {
                        min_dot = dot_a;
                        min_normal = Some(n);
                        min_point = Some(a1);
                    }
                    if dot_b < min_dot {
                        min_dot = dot_b;
                        min_normal = Some(n);
                        min_point = Some(b1);
                    }
                }
                let normal = min_normal.unwrap();
                return CollisionData::Collision(normal.normalize(), min_point.unwrap());
            }
        }
        unreachable!()
    }

    fn face_face(face1: Self, face2: Self) -> CollisionData {
        if let Self::Triangle(a1, b1, c1) = face1 {
            if let Self::Triangle(a2, b2, c2) = face2 {
                let normal1 = (a1 - c1).cross(b1 - c1);
                let normal2 = (a2 - c2).cross(b2 - c2);

                let mut min_dot_1 = f64::INFINITY;
                let mut min_dot_2 = f64::INFINITY;
                let mut min_point_1 = None;
                let mut min_point_2 = None;
                let mut split_1 = false;
                let mut split_2 = false;
                let mut sign_1 = 0.0;
                let mut sign_2 = 0.0;
                
                for point in [a1, b1, c1] {
                    let dot = normal2.dot(point - c2);
                    if dot.abs() < min_dot_1 {
                        min_dot_1 = dot.abs();
                        min_point_1 = Some(point);
                    }
                    if sign_1 == 0.0 {
                        sign_1 = dot.signum();
                    } else if dot.signum() != sign_1 {
                        split_1 = true;
                    }
                }
                for point in [a2, b2, c2] {
                    let dot = normal1.dot(point - c1);
                    if dot.abs() < min_dot_2 {
                        min_dot_2 = dot.abs();
                        min_point_2 = Some(point);
                    }
                    if sign_2 == 0.0 {
                        sign_2 = dot.signum();
                    } else if dot.signum() != sign_2 {
                        split_2 = true;
                    }
                }

                if split_1 && !split_2 {
                    return CollisionData::Collision(normal1.normalize(), min_point_2.unwrap());
                }
                if !split_1 && split_2 {
                    return CollisionData::Collision(normal2.normalize(), min_point_1.unwrap());
                }
                if min_dot_1 < min_dot_2 {
                    return CollisionData::Collision(normal2.normalize(), min_point_1.unwrap());
                }
                else {
                    return CollisionData::Collision(normal1.normalize(), min_point_2.unwrap());
                }
            }
        }
        unreachable!()
    }

    fn face_tet(face: Self, tet: Self) -> CollisionData {
        if let Self::Triangle(a1, b1, c1) = face {
            if let Self::Tetrahedron(a2, b2, c2, d2) = tet {
                let mut normals_pos = [
                    ((a2 - c2).cross(b2 - c2), c2),
                    ((a2 - d2).cross(b2 - d2), d2),
                    ((a2 - d2).cross(c2 - d2), d2),
                    ((b2 - d2).cross(c2 - d2), d2),
                ];
                if normals_pos[0].0.dot(d2) > 0.0 {
                    normals_pos[0].0 *= -1.0;
                }
                if normals_pos[1].0.dot(c2) > 0.0 {
                    normals_pos[1].0 *= -1.0;
                }
                if normals_pos[2].0.dot(b2) > 0.0 {
                    normals_pos[2].0 *= -1.0;
                }
                if normals_pos[3].0.dot(a2) > 0.0 {
                    normals_pos[3].0 *= -1.0;
                }
                let tri_normal = (a1 - c1).cross(b1 - c1);

                let mut is_sliced = [false, false, false, false];
                let mut is_outside = [false, false, false];
                for (i, (n, p)) in normals_pos.iter().enumerate() {
                    let dot_a = n.dot(a1 - p);
                    let dot_b = n.dot(b1 - p);
                    let dot_c = n.dot(c1 - p);
                    if dot_a.signum() != dot_b.signum() || dot_b.signum() != dot_c.signum() {
                        // The tet triangle is sliced
                        is_sliced[i] = true;
                    }
                    if dot_a > 0.0 {
                        is_outside[0] = true;
                    }
                    if dot_b > 0.0 {
                        is_outside[1] = true;
                    }
                    if dot_c > 0.0 {
                        is_outside[2] = true;
                    }
                }
                let tri_sliced = {
                    let dot_a = tri_normal.dot(a2 - c1);
                    let dot_b = tri_normal.dot(b2 - c1);
                    let dot_c = tri_normal.dot(c2 - c1);
                    let dot_d = tri_normal.dot(d2 - c1);
                    dot_a.signum() != dot_b.signum() || dot_b.signum() != dot_c.signum() || dot_c.signum() != dot_d.signum()
                };
                let num_tet_sliced = is_sliced[0] as u32 + is_sliced[1] as u32 + is_sliced[2] as u32 + is_sliced[3] as u32;

                // If the whole triangle is inside
                if !is_outside[0] && !is_outside[1] && !is_outside[2] {
                    let mut min_dot = f64::INFINITY;
                    let mut min_point = None;
                    for (_, p) in normals_pos {
                        let dot = tri_normal.dot(p - c1).abs();
                        if dot < min_dot {
                            min_dot = dot;
                            min_point = Some(p);
                        }
                    }
                    return CollisionData::Collision(tri_normal.normalize(), min_point.unwrap());
                }

                // If the triangle is partially inside
                if !is_outside[0] || !is_outside[1] || !is_outside[2] {
                    for (i, sliced) in is_sliced.iter().enumerate() {
                        if *sliced {
                            // Find out if this tet triangle slices the solo triangle
                            let tet_tri = match i {
                                0 => [a2, b2, c2],
                                1 => [a2, b2, d2],
                                2 => [a2, c2, d2],
                                3 => [b2, c2, d2],
                                _ => unreachable!()
                            };
                            let dot_0 = tri_normal.dot(tet_tri[0] - c1);
                            let dot_1 = tri_normal.dot(tet_tri[1] - c1);
                            let dot_2 = tri_normal.dot(tet_tri[2] - c1);
                            if dot_0.signum() != dot_1.signum() || dot_1.signum() != dot_2.signum() {
                                // They slice each other
                                let (normal, pos) = normals_pos[i];
                                let dot_a = normal.dot(a1 - pos).abs();
                                let dot_b = normal.dot(b1 - pos).abs();
                                let dot_c = normal.dot(c1 - pos).abs();
                                let min_point = if dot_a < dot_b && dot_a < dot_c {
                                    a1
                                } else if dot_b < dot_a && dot_b < dot_c {
                                    b1
                                } else {
                                    c1
                                };
                                return CollisionData::Collision(normal.normalize(), min_point);
                            }
                        }
                    }
                }

                // If there is no intersection
                if !tri_sliced || (tri_sliced && num_tet_sliced <= 2) {
                    // The triangle fully slices through
                    let mut min_dot = f64::INFINITY;
                    let mut min_point = None;
                    let mut min_normal = None;
                    for (n, p) in normals_pos {
                        let dot_a = n.dot(a1 - p).abs();
                        let dot_b = n.dot(b1 - p).abs();
                        let dot_c = n.dot(c1 - p).abs();
                        if dot_a < min_dot {
                            min_dot = dot_a;
                            min_point = Some(a1);
                            min_normal = Some(n);
                        }
                        if dot_b < min_dot {
                            min_dot = dot_b;
                            min_point = Some(b1);
                            min_normal = Some(n);
                        }
                        if dot_c < min_dot {
                            min_dot = dot_c;
                            min_point = Some(c1);
                            min_normal = Some(n);
                        }
                    }
                    return CollisionData::Collision(min_normal.unwrap().normalize(), min_point.unwrap());
                }

                // The triangle fully slices through
                let mut min_dot = f64::INFINITY;
                let mut min_point = None;
                for (_, p) in normals_pos {
                    let dot = tri_normal.dot(p - c1).abs();
                    if dot < min_dot {
                        min_dot = dot;
                        min_point = Some(p);
                    }
                }
                return CollisionData::Collision(tri_normal.normalize(), min_point.unwrap());
            }
        }
        unreachable!()
    }

    fn tet_tet(tet1: Self, tet2: Self) -> CollisionData {
        if let Self::Tetrahedron(a1, b1, c1, d1) = tet1 {
            if let Self::Tetrahedron(a2, b2, c2, d2) = tet2 {
                // This is very sussy. There's probably many more edge cases than this smallest-dot-product method.
                let normals_pos1 = [
                    ((a1 - c1).cross(b1 - c1), c1),
                    ((a1 - d1).cross(b1 - d1), d1),
                    ((a1 - d1).cross(c1 - d1), d1),
                    ((b1 - d1).cross(c1 - d1), d1),
                ];
                let normals_pos2 = [
                    ((a2 - c2).cross(b2 - c2), c2),
                    ((a2 - d2).cross(b2 - d2), d2),
                    ((a2 - d2).cross(c2 - d2), d2),
                    ((b2 - d2).cross(c2 - d2), d2),
                ];
                let mut min_dot = f64::INFINITY;
                let mut min_normal = None;
                let mut min_point = None;
                for (n, p) in normals_pos1 {
                    for p2 in [a2, b2, c2, d2] {
                        let dot = n.dot(p2 - p);
                        if dot < min_dot {
                            min_dot = dot;
                            min_normal = Some(n);
                            min_point = Some(p2);
                        }
                    }
                }
                for (n, p) in normals_pos2 {
                    for p2 in [a1, b1, c1, d1] {
                        let dot = n.dot(p2 - p);
                        if dot < min_dot {
                            min_dot = dot;
                            min_normal = Some(n);
                            min_point = Some(p2);
                        }
                    }
                }
                return CollisionData::Collision(min_normal.unwrap().normalize(), min_point.unwrap());
            }
        }
        unreachable!()
    }
}

pub enum CollisionData {
    NoCollision,
    Collision(Vector3<f64>, Vector3<f64>), // Normal, pos
}