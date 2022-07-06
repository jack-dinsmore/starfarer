use std::path::Path;
use lepton::prelude::*;
use std::rc::Rc;
use cgmath::{Vector3, Quaternion};
use byteorder::{LittleEndian, ReadBytesExt};

pub struct Ship {

}

impl Ship {
    pub fn _from_path(graphics: &Graphics, model_shader: &Shader<builtin::ModelSignature>, path: &Path) -> Object {
        let bytes = lepton::tools::read_as_bytes(path);
        Self::from_bytes(graphics, model_shader, &bytes)
    }

    pub fn from_bytes(graphics: &Graphics, model_shader: &Shader<builtin::ModelSignature>, bytes: &[u8]) -> Object {
        let (info_size, texture_size, num_indices) = (
            (&bytes[0..8]).read_u32::<LittleEndian>().unwrap() as usize,
            (&bytes[8..16]).read_u32::<LittleEndian>().unwrap() as usize,
            (&bytes[16..24]).read_u32::<LittleEndian>().unwrap() as usize,
        );

        let _info_bytes = &bytes[12..(info_size + 24)];
        let texture_bytes = &bytes[(info_size + 24)..(info_size + texture_size + 24)];
        let object_bytes = &bytes[(info_size + texture_size + 24)..];

        let ship_model = Rc::new(Model::new(graphics, &model_shader,
            VertexType::<vertex::VertexModel>::Compiled(object_bytes, num_indices),
            TextureType::Transparency(texture_bytes))
            .expect("Model creation failed"));

        let mut obj = Object::new(Vector3::new(0.0, 0.0, 0.0), Quaternion::new(1.0, 0.0, 0.0, 0.0));
        obj.add_model(ship_model.clone());
        obj
    }
}