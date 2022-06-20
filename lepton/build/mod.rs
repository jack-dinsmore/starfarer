mod fonts {
    use fontdue::Font;
    use std::fs::{self, File};
    use std::io::Read;
    use std::path::Path;

    const FONTS: &'static [(&'static str, usize)] = &[
        ("../assets/fonts/Roboto-Regular.ttf", 48)
    ];
    const N_ROWS: usize = 8;
    const N_COLS: usize = 12;

    fn load_font(font_path: &str, size: usize) {
        let font_path = Path::new(font_path);
        let mut f = File::open(font_path).expect("No file found");
        let metadata = fs::metadata(&font_path).expect("Unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        let font = Font::from_bytes(buffer, fontdue::FontSettings::default()).unwrap();

        let mut max_width = 0;
        let mut max_height = 0;
        let mut rasters = Vec::with_capacity(128 - 32);
        for i in 32..128 {
            let (metrics, bitmap) = font.rasterize(char::from_u32(i).unwrap(), size as f32);
            max_width = usize::max(max_width, metrics.width);
            max_height = usize::max(max_height, metrics.height);
            rasters.push((metrics, bitmap));
        }

        let (image_width, image_height) = (N_COLS * max_width, N_ROWS * max_height);
        let mut buffer = vec![0; image_width * image_height * 4];
        let mut font_index = 0;
        for row in 0..N_ROWS {
            for col in 0..N_COLS {
                for x in 0..rasters[font_index].0.width {
                    for y in 0..rasters[font_index].0.height {
                        let buffer_x = col * max_width + x;
                        let buffer_y = image_height - (row * max_height + y) - 1;
                        let bitmap_index = rasters[font_index].0.width * y + x;
                        let buffer_index = (buffer_y * image_width + buffer_x) * 4;
                        buffer[buffer_index + 3] = rasters[font_index].1[bitmap_index] as u8;
                    }
                }
                font_index += 1;
            }
        }

        image::save_buffer(
            font_path.parent().unwrap().join(format!("rendered/{}-{}.png", font_path.file_stem().expect("No filename").to_str().expect("Invalid filename"), size)),
            &buffer,
            image_width as u32,
            image_height as u32,
            image::ColorType::Rgba8
        ).expect("Could not save file");
    }

    pub fn load_fonts() {
        for (font, size) in FONTS {
            load_font(font, *size);
        }
    }
}

fn main() {
    fonts::load_fonts();
}