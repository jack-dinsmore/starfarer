/// These are copies of lepton functions so that lepton does not have to be a dependency.
use anyhow::{Result, bail};
use std::path::Path;
use std::fs::File;
use std::io::{Read, Cursor};
use crate::common::*;

pub fn struct_as_bytes<T>(s: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            (s as *const T) as *const u8,
            ::std::mem::size_of::<T>(),
        )
    }
}

pub fn read_as_bytes(path: &Path) -> Result<Vec<u8>> {
    let mut f = File::open(&path)?;
    let metadata = std::fs::metadata(&path).expect("Metadata was corrupt");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("Buffer was too short");

    Ok(buffer)
}

pub fn load_obj(path: &Path) -> Result<(Vec<VertexModel>, Vec<u32>)> {
    let model_obj = match tobj::load_obj(path, &tobj::LoadOptions{single_index: true, ..Default::default()}) {
        Ok(m) => m,
        Err(_) => bail!("Failed to load model object {}", path.display())
    };

    let mut vertices = vec![];
    let mut indices = vec![];

    let (models, _) = model_obj;
    for m in models.iter() {
        let mesh = &m.mesh;

        if mesh.texcoords.len() == 0 {
            bail!("Missing texture coordinates");
        }

        let total_vertices_count = mesh.positions.len() / 3;
        for i in 0..total_vertices_count {
            let vertex = VertexModel {
                pos: [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                ],
                normal: [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ],
                tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
            };
            vertices.push(vertex);
        }

        indices = mesh.indices.clone();
    }

    Ok((vertices, indices))
}

pub fn load_font_to_binary(font_path_name: &Path, size: usize) -> (Vec<u8>, Vec<u8>) {
    let font_path = std::path::Path::new(font_path_name);
    let mut f = std::fs::File::open(&font_path).expect("No file found");
    let metadata = std::fs::metadata(&font_path).expect("Unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let font = fontdue::Font::from_bytes(buffer, fontdue::FontSettings::default()).expect("Could not load font");

    let mut max_width = 0;
    let mut max_height = 0;
    let mut baseline = 0;
    let mut rasters = Vec::with_capacity(N_CHARS);
    for i in 32..128 {
        let (metrics, bitmap) = font.rasterize(char::from_u32(i).expect("Could not rasterize character"), size as f32);
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
                (left as u8 + 32) as char, (right as u8 + 32) as char, size as f32) {
                    Some(f) => (f * size as f32) as i32,
                None => 0,
            } - delta_width;
            if kern < i8::MIN as i32 || kern > i8::MAX as i32 {
                panic!("Kern {} was not in the right range", kern)
            }
            kern_array[left * N_CHARS + right] = kern as i8;
        }
    }

    let mut cursor = Cursor::new(Vec::new());

    image::write_buffer_with_format(
        &mut cursor,
        &buffer,
        image_width as u32,
        image_height as u32,
        image::ColorType::L8,
        image::ImageFormat::Png,
    ).expect("Could not write file data");

    let kern_bytes = kern_array.iter().map(|i| { unsafe { std::mem::transmute(*i) } }).collect::<Vec<u8>>();
    (cursor.into_inner(), kern_bytes)
}