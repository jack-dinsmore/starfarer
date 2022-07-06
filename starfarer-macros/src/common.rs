pub const N_COLS: usize = 12;
pub const N_ROWS: usize = 8;
pub const N_CHARS: usize = N_COLS * N_ROWS;

pub struct VertexModel {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}