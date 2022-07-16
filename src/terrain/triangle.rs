use cgmath::{Vector3, InnerSpace};
use lepton::prelude::*;


const PHI: f64 = 1.61803398875;
const ICO_VERTICES: [Vector3<f64>; 12] = [
    Vector3::new(0.0, 1.0, PHI), // 0
    Vector3::new(0.0, -1.0, PHI),
    Vector3::new(0.0, -1.0, -PHI), // 2
    Vector3::new(0.0, 1.0, -PHI),

    Vector3::new(1.0, PHI, 0.0),// 4
    Vector3::new(-1.0, PHI, 0.0),
    Vector3::new(-1.0, -PHI, 0.0), // 6
    Vector3::new(1.0, -PHI, 0.0),

    Vector3::new(PHI, 0.0, 1.0),// 8
    Vector3::new(PHI, 0.0, -1.0),
    Vector3::new(-PHI, 0.0, -1.0), // a
    Vector3::new(-PHI, 0.0, 1.0),
];
const ICO_INDICES: [[usize; 3]; 20] = [
    [0x0, 0x1, 0x8], // 0
    [0x1, 0x0, 0xb],
    [0x2, 0x3, 0x9],
    [0x3, 0x2, 0xa],

    [0x4, 0x5, 0x0], // 4
    [0x5, 0x4, 0x3],
    [0x6, 0x7, 0x1],
    [0x7, 0x6, 0x2],

    [0x8, 0x9, 0x4], // 8
    [0x9, 0x8, 0x7],
    [0xa, 0xb, 0x5],
    [0xb, 0xa, 0x6],

    [0x0, 0x8, 0x4], // 12
    [0x0, 0x5, 0xb],
    [0x1, 0xb, 0x6],
    [0x1, 0x7, 0x8],

    [0x3, 0x9, 0x4],
    [0x3, 0x5, 0xa],
    [0x2, 0x6, 0xa], // 16
    [0x2, 0x9, 0x7],
];
const NORMALS: [Vector3<f64>; 20] = [
    (ICO_VERTICES[ICO_INDICES[0][0]] + ICO_VERTICES[ICO_INDICES[0][1]] + ICO_VERTICES[ICO_INDICES[0][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[1][0]] + ICO_VERTICES[ICO_INDICES[1][1]] + ICO_VERTICES[ICO_INDICES[1][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[2][0]] + ICO_VERTICES[ICO_INDICES[2][1]] + ICO_VERTICES[ICO_INDICES[2][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[3][0]] + ICO_VERTICES[ICO_INDICES[3][1]] + ICO_VERTICES[ICO_INDICES[3][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[4][0]] + ICO_VERTICES[ICO_INDICES[4][1]] + ICO_VERTICES[ICO_INDICES[4][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[5][0]] + ICO_VERTICES[ICO_INDICES[5][1]] + ICO_VERTICES[ICO_INDICES[5][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[6][0]] + ICO_VERTICES[ICO_INDICES[6][1]] + ICO_VERTICES[ICO_INDICES[6][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[7][0]] + ICO_VERTICES[ICO_INDICES[7][1]] + ICO_VERTICES[ICO_INDICES[7][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[8][0]] + ICO_VERTICES[ICO_INDICES[8][1]] + ICO_VERTICES[ICO_INDICES[8][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[9][0]] + ICO_VERTICES[ICO_INDICES[9][1]] + ICO_VERTICES[ICO_INDICES[9][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[10][0]] + ICO_VERTICES[ICO_INDICES[10][1]] + ICO_VERTICES[ICO_INDICES[10][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[11][0]] + ICO_VERTICES[ICO_INDICES[11][1]] + ICO_VERTICES[ICO_INDICES[11][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[12][0]] + ICO_VERTICES[ICO_INDICES[12][1]] + ICO_VERTICES[ICO_INDICES[12][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[13][0]] + ICO_VERTICES[ICO_INDICES[13][1]] + ICO_VERTICES[ICO_INDICES[13][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[14][0]] + ICO_VERTICES[ICO_INDICES[14][1]] + ICO_VERTICES[ICO_INDICES[14][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[15][0]] + ICO_VERTICES[ICO_INDICES[15][1]] + ICO_VERTICES[ICO_INDICES[15][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[16][0]] + ICO_VERTICES[ICO_INDICES[16][1]] + ICO_VERTICES[ICO_INDICES[16][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[17][0]] + ICO_VERTICES[ICO_INDICES[17][1]] + ICO_VERTICES[ICO_INDICES[17][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[18][0]] + ICO_VERTICES[ICO_INDICES[18][1]] + ICO_VERTICES[ICO_INDICES[18][2]]) / 3.0,
    (ICO_VERTICES[ICO_INDICES[19][0]] + ICO_VERTICES[ICO_INDICES[19][1]] + ICO_VERTICES[ICO_INDICES[19][2]]) / 3.0,
];

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct MapID {
    face: u8,
    map: u8,
}

pub struct TerrainSettings {
    pub face_subdivision: u32,
    pub map_subdivision: u32,
    pub height_subdivision: u32,
    pub height: f64,
}

pub struct Triangle<'a, F: Fn([f64; 3]) -> f64> {
    id: MapID,
    degree: u8,
    settings: &'a TerrainSettings,
    height_fn: F,
}

impl<'a, F: Fn([f64; 3]) -> f64> Triangle<'a, F> {
    pub fn new(id: MapID, degree: u8, settings: &'a TerrainSettings, height_fn: F) -> Self {
        Self {
            id,
            degree,
            settings,
            height_fn
        }
    }

    pub fn load_new(&self) -> (Vec<vertex::VertexLP>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let num_faces = self.settings.map_subdivision * self.settings.map_subdivision;
        let num_points = (self.settings.map_subdivision + 1) * (self.settings.map_subdivision + 1);
        let mut top_points = Vec::with_capacity(num_points as usize);
        let mut bottom_points = Vec::with_capacity(num_points as usize);
        
        let poses = self.get_pos_map();
        self.get_points(0, &poses, &mut top_points);
        self.get_points(1, &poses, &mut bottom_points);

        for height_index in 0..self.settings.height_subdivision {
            let mut row_index = 0;
            let top_radius = (height_index + 1) as f64 * self.settings.height / self.settings.height_subdivision as f64;
            let bottom_radius = height_index as f64 * self.settings.height / self.settings.height_subdivision as f64;

            for face_index in 0..num_faces as usize {
                if face_index >= (row_index + 1) * (row_index + 1) {
                    row_index += 1;
                }
                let in_index = face_index - row_index * row_index;
                let start_point_up = row_index * (row_index + 1) / 2;
                let start_point_down = row_index * (row_index + 1) / 2;

                let corner_vals = match in_index % 2 {
                    0 => [
                        top_points[start_point_up + in_index / 2 + 1],
                        top_points[start_point_up + in_index / 2],
                        top_points[start_point_down + in_index / 2 + 1],
                        bottom_points[start_point_up + in_index / 2 + 1],
                        bottom_points[start_point_up + in_index / 2],
                        bottom_points[start_point_down + in_index / 2 + 1],
                    ],
                    1 => [
                        top_points[start_point_up + in_index / 2],
                        top_points[start_point_down + in_index / 2],
                        top_points[start_point_down + in_index / 2 + 1],
                        bottom_points[start_point_up + in_index / 2],
                        bottom_points[start_point_down + in_index / 2],
                        bottom_points[start_point_down + in_index / 2 + 1],
                    ],
                    _ => unreachable!()
                };
                let corner_poses = match in_index % 2 {
                    0 => [
                        top_radius * poses[start_point_up + in_index / 2 + 1],
                        top_radius * poses[start_point_up + in_index / 2],
                        top_radius * poses[start_point_down + in_index / 2 + 1],
                        bottom_radius * poses[start_point_up + in_index / 2 + 1],
                        bottom_radius * poses[start_point_up + in_index / 2],
                        bottom_radius * poses[start_point_down + in_index / 2 + 1],
                    ],
                    1 => [
                        top_radius * poses[start_point_up + in_index / 2],
                        top_radius * poses[start_point_down + in_index / 2],
                        top_radius * poses[start_point_down + in_index / 2 + 1],
                        bottom_radius * poses[start_point_up + in_index / 2],
                        bottom_radius * poses[start_point_down + in_index / 2],
                        bottom_radius * poses[start_point_down + in_index / 2 + 1],
                    ],
                    _ => unreachable!()
                };
                super::triangulation::assess_prism(corner_poses, corner_vals, &mut vertices, &mut indices);
            }
            std::mem::swap(&mut bottom_points, &mut top_points);
            top_points.clear();
            self.get_points(height_index + 1, &poses, &mut top_points);
        }

        (vertices, indices)
    }

    pub fn get_id(pos: &Vector3<f64>) -> MapID {
        let mut max_dot = 0;
        let mut max_index = 0;
        for (i, normal) in NORMALS.iter().enumerate() {
            let d = normal.dot(pos);
            if d > max_dot {
                max_dot = d;
                max_index = i;
            }
        }
        
        // Get map id within the given face
    }
}

impl<'a, F: Fn([f64; 3]) -> f64> Triangle<'a, F> {
    fn get_points(&self, height_index: u32, pos_map: &Vec<Vector3<f64>>, target: &mut Vec<f64>) {
        let mut vertex = 0;
        for row_num in 0.. {
            for in_num in 0..row_num {
                target.push((self.height_fn)([pos_map[vertex].x, pos_map[vertex].y, pos_map[vertex].z]));
                vertex += 1;
            }
        }
    }

    fn get_pos_map(&self) -> Vec<Vector3<f64>> {
        let line_vertices = (self.settings.map_subdivision + 1);
        let mut poses = Vec::with_capacity((line_vertices * line_vertices) as usize);
        let corners = self.get_corners();

        for row_num in 0.. {
            for in_num in 0..row_num {
                let bary_x = 1.0 - row_num as f64 / line_vertices as f64;
                let bary_z = in_num as f64 / line_vertices as f64;
                let bary_y = 1.0 - bary_x - bary_z;
                poses.push(Vector3::new(
                    corners[0][0] * bary_x + corners[1][0] * bary_y + corners[2][0] * bary_z,
                    corners[0][1] * bary_x + corners[1][1] * bary_y + corners[2][1] * bary_z,
                    corners[0][2] * bary_x + corners[1][2] * bary_y + corners[2][2] * bary_z,
                ));
            }
        }
        poses
    }

    fn get_corners(&self) -> [Vector3<f64>; 3] {
        let mut row = 0;
        loop {
            if (row + 1) * (row + 1) > self.id.map {
                break;
            }
            row += 1;
        }
        let bary_x_top = 1.0 - row as f64 / self.settings.face_subdivision as f64;
        let bary_x_bottom = 1.0 -(row + 1) as f64 / self.settings.face_subdivision as f64;
        let face_in = self.id.map - row * row;
        let baries = match face_in % 2 {// [[Bary x, bary z]]. 
            0 => [
                [bary_x_top, (face_in / 2) as f64 / self.settings.face_subdivision as f64],
                [bary_x_bottom, (face_in / 2) as f64 / self.settings.face_subdivision as f64],
                [bary_x_bottom, (face_in / 2 + 1) as f64 / self.settings.face_subdivision as f64],
            ],
            1 => [
                [bary_x_top, (face_in / 2 + 1) as f64 / self.settings.face_subdivision as f64],
                [bary_x_top, (face_in / 2) as f64 / self.settings.face_subdivision as f64],
                [bary_x_bottom, (face_in / 2 + 1) as f64 / self.settings.face_subdivision as f64],
            ],
            _ => unreachable!()
        };
        [
            (ICO_VERTICES[ICO_INDICES[self.id.face as usize][0]] * baries[0][0] + 
            ICO_VERTICES[ICO_INDICES[self.id.face as usize][1]] * (1.0 - baries[0][0] - baries[0][1]) + 
            ICO_VERTICES[ICO_INDICES[self.id.face as usize][2]] * baries[0][1]).normalize(),

            (ICO_VERTICES[ICO_INDICES[self.id.face as usize][0]] * baries[1][0] + 
            ICO_VERTICES[ICO_INDICES[self.id.face as usize][1]] * (1.0 - baries[1][0] - baries[1][1]) + 
            ICO_VERTICES[ICO_INDICES[self.id.face as usize][2]] * baries[1][1]).normalize(),

            (ICO_VERTICES[ICO_INDICES[self.id.face as usize][0]] * baries[2][0] + 
            ICO_VERTICES[ICO_INDICES[self.id.face as usize][1]] * (1.0 - baries[2][0] - baries[2][1]) + 
            ICO_VERTICES[ICO_INDICES[self.id.face as usize][2]] * baries[2][1]).normalize()
        ]
    }
}