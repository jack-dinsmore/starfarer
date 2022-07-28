use cgmath::{Vector3, InnerSpace};
use lepton::prelude::*;
use super::LoadDegree;

const ADJACENCIES: [[u8; 4]; 6] = [
    // Top, right, bottom, left
    [5, 2, 4, 3],
    [5, 2, 4, 3],
    [1, 4, 0, 5],
    [1, 4, 0, 5],
    [3, 0, 2, 1,],
    [3, 0, 2, 1,],
];

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct MapID {
    pub face: u8,
    pub map_row: u8,
    pub map_col: u8,
}

impl MapID {
    pub fn dist(a: MapID, b: MapID, face_subdivision: u8) -> i32 {
        if a.face == b.face {
            (a.map_row as i8 - b.map_row as i8).abs().max(
                (a.map_col as i8 - b.map_col as i8).abs()) as i32
        } else {
            let a_dist = if ADJACENCIES[a.face as usize][0] == b.face {
                a.edge_dist(0, face_subdivision)
            } else if ADJACENCIES[a.face as usize][1] == b.face {
                a.edge_dist(1, face_subdivision)
            } else if ADJACENCIES[a.face as usize][2] == b.face {
                a.edge_dist(2, face_subdivision)
            } else if ADJACENCIES[a.face as usize][3] == b.face {
                a.edge_dist(3, face_subdivision)
            } else {
                return i32::MAX;
            };
            let b_dist = if ADJACENCIES[b.face as usize][0] == a.face {
                b.edge_dist(0, face_subdivision)
            } else if ADJACENCIES[b.face as usize][1] == a.face {
                b.edge_dist(1, face_subdivision)
            } else if ADJACENCIES[b.face as usize][2] == a.face {
                b.edge_dist(2, face_subdivision)
            } else if ADJACENCIES[b.face as usize][3] == a.face {
                b.edge_dist(3, face_subdivision)
            } else {
                return i32::MAX;
            };
            a_dist + b_dist + 1
        }
    }

    fn edge_dist(&self, edge: u8, face_subdivision: u8) -> i32 {
        match edge {
            0 => self.map_col as i32, // Top,
            1 => face_subdivision as i32 - self.map_row as i32 - 1, // Right,
            2 => face_subdivision as i32 - self.map_col as i32 - 1, // Bottom,
            3 => self.map_row as i32, // Left,
            _ => unreachable!()
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PlanetSettings {
    pub face_subdivision: u32,
    pub map_subdivision: u32,
    pub height_subdivision: u32,
    pub height: f64,
    pub radius: f64,
}

pub struct Square<F: Fn(Vector3<f64>) -> f64> {
    id: MapID,
    degree: LoadDegree,
    settings: PlanetSettings,
    value_fn: F,
}

impl<F: Fn(Vector3<f64>) -> f64> Square<F> {
    pub fn new(id: MapID, degree: LoadDegree, settings: PlanetSettings, value_fn: F) -> Self {
        Self {
            id,
            degree,
            settings,
            value_fn
        }
    }

    pub fn load_new(&self) -> (Vec<vertex::VertexLP>, Vec<u32>) {
        let mut vertices = Vec::new();
        let map_subdivision = self.settings.map_subdivision >> (self.degree as u8);
        let num_points = (map_subdivision + 1) * (map_subdivision + 1);
        let mut top_points = Vec::with_capacity(num_points as usize);
        let mut bottom_points = Vec::with_capacity(num_points as usize);
        let poses = self.get_pos_map();
        self.get_points(0, &poses, &mut bottom_points);

        for height_index in 0..self.settings.height_subdivision {
            self.get_points(height_index + 1, &poses, &mut top_points);
            let top_radius = self.settings.radius * (
                1.0 - self.settings.height / 2.0 + (height_index + 1) as f64 * self.settings.height / self.settings.height_subdivision as f64);
            let bottom_radius = self.settings.radius * (
                1.0 - self.settings.height / 2.0 + height_index as f64 * self.settings.height / self.settings.height_subdivision as f64);
            for row_index in 0..map_subdivision {
                for col_index in 0..map_subdivision {
                    let corner_poses = [
                        bottom_radius * poses[(row_index * (map_subdivision + 1) + col_index) as usize],
                        bottom_radius * poses[(row_index * (map_subdivision + 1) + col_index + 1) as usize],
                        bottom_radius * poses[((row_index + 1) * (map_subdivision + 1) + col_index + 1) as usize],
                        bottom_radius * poses[((row_index + 1) * (map_subdivision + 1) + col_index) as usize],
                        top_radius * poses[(row_index * (map_subdivision + 1) + col_index) as usize],
                        top_radius * poses[(row_index * (map_subdivision + 1) + col_index + 1) as usize],
                        top_radius * poses[((row_index + 1) * (map_subdivision + 1) + col_index + 1) as usize],
                        top_radius * poses[((row_index + 1) * (map_subdivision + 1) + col_index) as usize],
                    ];
                    let corner_vals = [
                        bottom_points[(row_index * (map_subdivision + 1) + col_index) as usize],
                        bottom_points[(row_index * (map_subdivision + 1) + col_index + 1) as usize],
                        bottom_points[((row_index + 1) * (map_subdivision + 1) + col_index + 1) as usize],
                        bottom_points[((row_index + 1) * (map_subdivision + 1) + col_index) as usize],
                        top_points[(row_index * (map_subdivision + 1) + col_index) as usize],
                        top_points[(row_index * (map_subdivision + 1) + col_index + 1) as usize],
                        top_points[((row_index + 1) * (map_subdivision + 1) + col_index + 1) as usize],
                        top_points[((row_index + 1) * (map_subdivision + 1) + col_index) as usize],
                    ];

                    super::triangulation::assess_cube(corner_poses, corner_vals, &mut vertices);
                }
            }
            std::mem::swap(&mut bottom_points, &mut top_points);
            top_points.clear();
        }
        let indices = (0..vertices.len() as u32).collect::<Vec<_>>();
        (vertices, indices)
    }

    pub fn load_from_old(&self, _model: &Model, _old_degree: u8) -> (Vec<vertex::VertexLP>, Vec<u32>) {
        self.load_new()
        //// Implement this
    }
}

impl<F: Fn(Vector3<f64>) -> f64> Square<F> {
    fn get_points(&self, height_index: u32, pos_map: &[Vector3<f64>], target: &mut Vec<f64>) {
        let map_subdivision = self.settings.map_subdivision >> (self.degree as u8);
        let mut vertex = 0;
        let radius_frac = 1.0 - self.settings.height / 2.0 + height_index as f64 * self.settings.height / self.settings.height_subdivision as f64;
        for _row_num in 0..(map_subdivision + 1) {
            for _col_num in 0..(map_subdivision + 1) {
                target.push((self.value_fn)(radius_frac * pos_map[vertex]));
                vertex += 1;
            }
        }
    }

    fn get_pos_map(&self) -> Vec<Vector3<f64>> {
        let map_subdivision = self.settings.map_subdivision >> (self.degree as u8);
        let mut poses = Vec::with_capacity(((map_subdivision + 1) * (map_subdivision + 1)) as usize);
        let half_length = (self.settings.face_subdivision * map_subdivision) as f64 / 2.0;
        let offset_row = self.id.map_row as f64 * map_subdivision as f64 - half_length;
        let offset_col = self.id.map_col as f64 * map_subdivision as f64 - half_length;
        for row_num in 0..(map_subdivision + 1) {
            for col_num in 0..(map_subdivision + 1) {
                poses.push(match self.id.face {
                    0 => Vector3::new(half_length, offset_row + row_num as f64, offset_col + col_num as f64).normalize(),
                    1 => Vector3::new(-half_length, offset_row + (map_subdivision - row_num) as f64, offset_col + col_num as f64).normalize(),
                    2 => Vector3::new(offset_col + col_num as f64, half_length, offset_row + row_num as f64).normalize(),
                    3 => Vector3::new(offset_col + col_num as f64, -half_length, offset_row + (map_subdivision - row_num) as f64).normalize(),
                    4 => Vector3::new(offset_row + row_num as f64, offset_col + col_num as f64, half_length).normalize(),
                    5 => Vector3::new(offset_row + (map_subdivision - row_num) as f64, offset_col + col_num as f64, -half_length).normalize(),
                    _ => unreachable!()
                });
            }
        }
        poses
    }
}

pub fn get_id(pos: Vector3<f64>, face_subdivision: u32) -> MapID {
    let (face, normal_pos) = if pos.x.abs() > pos.y.abs() && pos.x.abs() > pos.z.abs() {
        if pos.x > 0.0 {
            (0, pos / pos.x)
        } else {
            (1, pos / -pos.x)
        }
    } else if pos.y.abs() > pos.x.abs() && pos.y.abs() > pos.z.abs() {
        if pos.y > 0.0 {
            (2, pos / pos.y)
        } else {
            (3, pos / -pos.y)
        }
    } else {
        if pos.z > 0.0 {
            (4, pos / pos.z)
        } else {
            (5, pos / -pos.z)
        }
    };
    let (row_entry, col_entry) = match face {
        0 => (normal_pos.y, normal_pos.z),
        1 => (normal_pos.y, normal_pos.z),
        2 => (normal_pos.z, normal_pos.x),
        3 => (normal_pos.z, normal_pos.x),
        4 => (normal_pos.x, normal_pos.y),
        5 => (normal_pos.x, normal_pos.y),
        _ => unreachable!()
    };
    let map_row = ((row_entry + 1.0) / 2.0 * face_subdivision as f64) as u8;
    let map_col = ((col_entry + 1.0) / 2.0 * face_subdivision as f64) as u8;
    // Get map id within the given face
    MapID { face, map_row, map_col }
}