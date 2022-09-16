use statrs::function::erf;

#[derive(Clone, Copy, Debug)]
pub struct PlanetSettings {
    pub face_subdivision: u32,
    pub map_subdivision: u32,
    pub height_subdivision: u32,
    pub height: f64,
    pub radius: f64,
    pub color_scheme: ColorScheme,
    pub spikiness: i32,
}

#[derive(Clone, Copy, Debug)]
pub enum ColorScheme {
    Single([f32; 3]),
    Double([f32; 3], [f32; 3], f64), // Upper color, Lower color, Height threshold
}

impl ColorScheme {
    pub fn make_double(c1: [f32; 3], c2: [f32; 3], color_frac: f64, avg_height: f64, stdev: f64) -> Self {
        let barrier = avg_height + 2.0f64.sqrt() * stdev * erf::erf_inv(2.0 * color_frac - 1.0);
        Self::Double(c1, c2, barrier)
    }

    pub fn get_color(&self, height: f64) -> &[f32; 3] {
        match self {
            Self::Single(c1) => c1,
            Self::Double(c1, c2, barrier) => 
                if height > *barrier {
                    c1
                } else {
                    c2
                },
        }
    }
}