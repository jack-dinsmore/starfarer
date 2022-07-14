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

#[derive(Serialize, Deserialize)]
pub struct ShipByteData {
    pub info_bytes: Vec<u8>,
    pub outside: (Vec<vertex::VertexLP>, Vec<u32>),
    pub inside: Option<(Vec<vertex::VertexLP>, Vec<u32>)>,
    pub transparent: Option<(Vec<vertex::VertexLP>, Vec<u32>)>,
}

pub struct Ship {
    pub object: Object,
    pub outside_model: Option<Rc<Model>>, // These are all options so they can be taken.
    pub inside_model: Option<Rc<Model>>,
    pub transparent_model: Option<Rc<Model>>,
    pub rigid_body: Option<RigidBody>,
    data: ShipData,
}

impl Ship {
    pub fn _from_path(graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>, object_manager: &mut ObjectManager, path: &Path) -> Ship {
        let bytes = lepton::tools::read_as_bytes(path).unwrap();
        Self::from_bytes(graphics, low_poly_shader, object_manager, &bytes)
    }

    pub fn from_bytes(graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>, object_manager: &mut ObjectManager, bytes: &[u8]) -> Ship {
        let byte_data: ShipByteData = bincode::deserialize(bytes).unwrap();

        let data: ShipData = bincode::deserialize(&byte_data.info_bytes).unwrap();

        let outside_model = Some(Rc::new(Model::new(graphics, &low_poly_shader,
            VertexType::<vertex::VertexLP>::Specified(byte_data.outside.0, byte_data.outside.1),
            TextureType::None)
            .expect("Outside model creation failed")));
        let inside_model = match byte_data.inside {
            Some(data) => Some(Rc::new(Model::new(graphics, &low_poly_shader,
                VertexType::<vertex::VertexLP>::Specified(data.0, data.1),
                TextureType::None)
                .expect("Inside model creation failed"))),
            None => None
        };
        let transparent_model = match byte_data.transparent {
            Some(data) => Some(Rc::new(Model::new(graphics, &low_poly_shader,
                VertexType::<vertex::VertexLP>::Specified(data.0, data.1),
                TextureType::None)
                .expect("Transparent model creation failed"))),
            None => None
        };
        let rigid_body = Some(RigidBody::new(Vector3::new(-5.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0)));
        let object = object_manager.get_object();
        Ship {
            object,
            outside_model,
            inside_model,
            transparent_model,
            rigid_body,
            data,
        }
    }

    pub fn get_models(&mut self) -> Vec<Rc<Model>> {
        let mut output = Vec::new();
        match self.outside_model.take() {
            Some(o) => output.push(o),
            None => panic!("Ship had no outside model")
        };
        match self.inside_model.take() {
            Some(o) => output.push(o),
            None => panic!("Ship had no inside model")
        };
        match self.transparent_model.take() {
            Some(o) => output.push(o),
            None => ()
        };
        output
    }
}