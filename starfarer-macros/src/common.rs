pub const N_COLS: usize = 12;
pub const N_ROWS: usize = 8;
pub const N_CHARS: usize = N_COLS * N_ROWS;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum Mesh {
    BumpedMesh(Vec<VertexLP>, Vec<u32>, Vec<u8>, f32),
    ModelMesh(Vec<VertexLP>, Vec<u32>),
    ColliderMesh(Vec<[f32; 3]>)
}

pub type ModelData = HashMap<String, Mesh>;

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VertexLP {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub info: [f32; 3],
}