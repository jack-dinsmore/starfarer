#[macro_export]
macro_rules! load_font {
    ( $font_path_name:literal, $rendered_dir: literal, $size:literal ) => {
        use std::io::{Read, Write};

        const N_ROWS: usize = 8;
        const N_COLS: usize = 12;
        const N_CHARS: usize = N_ROWS * N_COLS;

        let font_path = std::path::Path::new($font_path_name);
        let mut f = std::fs::File::open(&font_path).expect("No file found");
        let metadata = std::fs::metadata(&font_path).expect("Unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        let font = $crate::macros::fontdue::Font::from_bytes(buffer, $crate::macros::fontdue::FontSettings::default()).expect("Could not load font");

        let mut max_width = 0;
        let mut max_height = 0;
        let mut baseline = 0;
        let mut rasters = Vec::with_capacity(N_CHARS);
        for i in 32..128 {
            let (metrics, bitmap) = font.rasterize(char::from_u32(i).expect("Could not rasterize character"), $size as f32);
            max_width = usize::max(max_width, metrics.width);
            if metrics.height > max_height {
                max_height = metrics.height;
                baseline = -metrics.ymin;
            }
            rasters.push((metrics, bitmap));
        }

        let (image_width, image_height) = (N_COLS * max_width, N_ROWS * max_height);
        let mut buffer = vec![0; image_width * image_height];
        let mut font_index = 0;
        for row in 0..N_ROWS {
            for col in 0..N_COLS {
                for x in 0..rasters[font_index].0.width {
                    for flip_y in 0..rasters[font_index].0.height {
                        let buffer_x = (col * max_width) as isize + (x as isize + rasters[font_index].0.xmin as isize);
                        let buffer_x = if buffer_x < 0 { 0 } else { buffer_x as usize };
                        let buffer_x = if buffer_x >= image_width { image_width - 1 } else { buffer_x };

                        let buffer_y = (image_height - (row + 1) * max_height) as isize + (flip_y as isize + rasters[font_index].0.ymin as isize + baseline as isize);
                        let buffer_y = if buffer_y < 0 { 0 } else { buffer_y as usize };
                        let buffer_y = if buffer_y >= image_height { image_height - 1 } else { buffer_y };
                        
                        let bitmap_index = rasters[font_index].0.width * (rasters[font_index].0.height - flip_y - 1) + x;
                        let buffer_index = buffer_y * image_width + buffer_x;
                        buffer[buffer_index] = rasters[font_index].1[bitmap_index] as u8;
                    }
                }
                font_index += 1;
            }
        }

        // Kernings
        let mut kern_array = [0; N_CHARS * N_CHARS];
        for left in 0..N_CHARS {
            for right in 0..N_CHARS {
                let delta_width = max_width as i32 - rasters[left].0.width as i32;
                let kern = match font.horizontal_kern(
                    (left as u8 + 32) as char, (right as u8 + 32) as char, $size as f32) {
                        Some(f) => (f * $size as f32) as i32,
                        None => 0,
                    } - delta_width;
                    if kern < i8::MIN as i32 || kern > i8::MAX as i32 {
                        panic!("Kern {} was not in the right range", kern)
                    }
                    kern_array[left * N_CHARS + right] = kern as i8;
                }
            }
            let stem = font_path.file_stem().expect("No filename").to_str().expect("Invalid filename");
        let mut kern_file = std::fs::File::create(std::path::Path::new(&format!("{}/{}-{}.dat", $rendered_dir, stem, $size))).expect("Could not open kern file");
        let kern_bytes = unsafe {
            std::slice::from_raw_parts(
                kern_array.as_mut_ptr() as *mut u8,
                kern_array.len()
            )
        };
        kern_file.write_all(&kern_bytes).expect("Could not write to kern file");
        
        // Write image
        $crate::macros::image::save_buffer(
            std::path::Path::new(&format!("{}/{}-{}.png", $rendered_dir, stem, $size)),
            &buffer,
            image_width as u32,
            image_height as u32,
            $crate::macros::image::ColorType::L8
        ).expect("Could not save file");
    };
}