#[cfg(test)]
mod tests;

mod bytecode;
mod part;
mod fakes {
    use cgmath::{Vector3, Matrix3, Quaternion};
    use serde::{Serialize, Deserialize};
    
    #[derive(Serialize, Deserialize)]
    #[serde(remote = "Vector3::<f32>")]
    pub struct FakeVector {
        x: f32,
        y: f32,
        z: f32,
    }
    #[derive(Serialize, Deserialize)]
    #[serde(remote = "Matrix3::<f32>")]
    pub struct FakeMatrix {
        #[serde(with = "FakeVector")]
        x: Vector3<f32>,
        #[serde(with = "FakeVector")]
        y: Vector3<f32>,
        #[serde(with = "FakeVector")]
        z: Vector3<f32>,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "Quaternion::<f32>")]
    pub struct FakeQuaternion {
        #[serde(with = "FakeVector")]
        v: Vector3<f32>,
        s: f32,
    }
}

use lepton::prelude::*;
use cgmath::{Vector3, Matrix3, Quaternion};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::rc::Rc;
use byteorder::{LittleEndian, ReadBytesExt};

use fakes::*;
use part::*;


#[derive(Serialize, Deserialize)]
pub struct ShipData {
    id: PartID,
    #[serde(with = "FakeVector")]
    center_of_mass: Vector3<f32>,
    #[serde(with = "FakeMatrix")]
    moment_of_inertia: Matrix3<f32>,
    attachments: HashMap<PartID, PartInfo>,
}

pub struct Ship {
    pub object: Object,
    pub model: Option<Rc<Model>>,
    pub rigid_body: Option<RigidBody>,
    data: ShipData,
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

        let info_bytes = &bytes[12..(info_size + 24)];
        let texture_bytes = &bytes[(info_size + 24)..(info_size + texture_size + 24)];
        let object_bytes = &bytes[(info_size + texture_size + 24)..];

        let data: ShipData = bincode::deserialize(info_bytes).unwrap();

        let model = Some(Rc::new(Model::new(graphics, &model_shader,
            VertexType::<vertex::VertexModel>::Compiled(object_bytes, num_indices),
            TextureType::Transparency(texture_bytes))
            .expect("Model creation failed")));
        let rigid_body = Some(RigidBody::new(Vector3::new(-5.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)));
        let object = object_manager.get_object();
        Ship {
            object,
            model,
            rigid_body,
            data,
        }
    }
}