use std::path::Path;

const SIZE: usize = 100;

pub struct Sky {

}

impl Sky {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn save(&self, file_name: &str) {
        let mut buffer = [0; SIZE * SIZE * 4];
        let mut buffer_index = 0;
        for yi in 0..SIZE {
            for xi in 0..SIZE {
                let frac_x = xi as f32 / SIZE as f32 * 2.0 - 1.0;
                let frac_y = yi as f32 / SIZE as f32 * 2.0 - 1.0;
                let color = self.get_color(frac_x, frac_y);
                buffer[buffer_index + 0] = color[0];
                buffer[buffer_index + 1] = color[1];
                buffer[buffer_index + 2] = color[2];
                buffer[buffer_index + 3] = color[3];
                buffer_index += 4;
            }
        }
        image::save_buffer(&Path::new(file_name), &buffer, SIZE as u32, SIZE as u32, image::ColorType::Rgba8).unwrap();
    }
}

impl Sky {
    fn get_color(&self, frac_x: f32, frac_y: f32) -> [u8; 4] {
        // frac_x is the cosine of the angle between you and the sun, from the planet (0 is sunset)
        // frac_y is the cosine of the angle between look and the sun

        const G_PEAK: f32 = 0.85;
        const R_SLOPE: f32 = 0.75;
        const B_SLOPE: f32 = 0.6;

        let r: i32 = (0.0f32.max(frac_y * R_SLOPE * (1.0 - frac_x)) * 255.0) as i32;
        let g: i32 = (0.0f32.max((G_PEAK - (1.5 * frac_y - G_PEAK).abs()) * R_SLOPE * (1.0 - frac_x)) * 120.0) as i32;
        let brightness: i32 = (255.0 * B_SLOPE * (frac_x + 1.0)) as i32;

        let add = 0.max(brightness - 255);
        [
            (r + add).clamp(0, 255) as u8,
            (g + add).clamp(0, 255) as u8,
            (255 - r + add).clamp(0, 255) as u8,
            brightness.clamp(0, 255) as u8
        ]
    }
}