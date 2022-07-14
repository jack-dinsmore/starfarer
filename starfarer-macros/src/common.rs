pub const N_COLS: usize = 12;
pub const N_ROWS: usize = 8;
pub const N_CHARS: usize = N_COLS * N_ROWS;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

pub type ModelData = HashMap<String, (Vec<VertexLP>, Vec<u32>)>;

#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VertexLP {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
    pub info: [f32; 3],
}