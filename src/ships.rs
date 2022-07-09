use std::path::Path;
use lepton::prelude::*;
use std::rc::Rc;
use cgmath::{Vector3, Quaternion};
use byteorder::{LittleEndian, ReadBytesExt};

pub struct Ship {
    pub object: Object,
    pub model: Option<Rc<Model>>,
    pub rigid_body: Option<RigidBody>,
}

impl Ship {
    pub fn _from_path(graphics: &Graphics, model_shader: &Shader<builtin::ModelSignature>, object_manager: &mut ObjectManager, path: &Path) -> Ship {
        let bytes = lepton::tools::read_as_bytes(path).unwrap();
        Self::from_bytes(graphics, model_shader, object_manager, &bytes)
    }

    pub fn from_bytes(graphics: &Graphics, model_shader: &Shader<builtin::ModelSignature>, object_manager: &mut ObjectManager, bytes: &[u8]) -> Ship {
        let (info_size, texture_size, num_indices) = (
            (&bytes[0..8]).read_u32::<LittleEndian>().unwrap() as usize,
            (&bytes[8..16]).read_u32::<LittleEndian>().unwrap() as usize,
            (&bytes[16..24]).read_u32::<LittleEndian>().unwrap() as usize,
        );

        let _info_bytes = &bytes[12..(info_size + 24)];
        let texture_bytes = &bytes[(info_size + 24)..(info_size + texture_size + 24)];
        let object_bytes = &bytes[(info_size + texture_size + 24)..];

        let model = Some(Rc::new(Model::new(graphics, &model_shader,
            VertexType::<vertex::VertexModel>::Compiled(object_bytes, num_indices),
            TextureType::Transparency(texture_bytes))
            .expect("Model creation failed")));
        let rigid_body = Some(RigidBody::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.5, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0)));
        let object = object_manager.get_object();
        Ship {
            object,
            model,
            rigid_body,
        }
    }
}