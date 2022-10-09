use std::ffi::CStr;
use std::os::raw::c_char;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use anyhow::{Result, bail};

use crate::shader::vertex::VertexModel;

/// Helper function to convert [c_char; SIZE] to string
pub(crate) fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string.to_str().expect("Failed to convert vulkan raw string.").to_owned()
}

pub fn struct_as_bytes<T>(s: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            (s as *const T) as *const u8,
            ::std::mem::size_of::<T>(),
        )
    }
}

pub fn bytes_as_struct<T>(bytes: &[u8]) -> T {
    unsafe { std::ptr::read(bytes.as_ptr() as *const _) }
}

pub fn read_as_bytes(path: &Path) -> Result<Vec<u8>> {
    let mut f = File::open(&path)?;
    let metadata = fs::metadata(&path).expect("Metadata was corrupt");
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

        if mesh.texcoords.is_empty() {
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
                coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
            };
            vertices.push(vertex);
        }

        indices = mesh.indices.clone();
    }

    Ok((vertices, indices))
}