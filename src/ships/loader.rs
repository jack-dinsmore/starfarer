use lepton::prelude::*;
use lepton::prelude::vertex::VertexLP;
use std::collections::HashMap;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
use cgmath::{Vector3};

use super::primitives::*;
use starfarer_macros::include_model;

pub struct ShipLoader {
    models: HashMap<PartID, HashMap<String, Rc<Model>>>,
    paths: HashMap<PartID, String>,
}

impl ShipLoader {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    pub fn purge(&mut self) {
        self.models.clear();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Mesh {
    BumpedMesh(Vec<VertexLP>, Vec<u32>, Vec<u8>, f32),
    ModelMesh(Vec<VertexLP>, Vec<u32>),
    ColliderMesh(Vec<[f32; 3]>)
}

impl ShipLoader {
    /// Returns None if the model is known not to exist, and Some if the model does exist. Loads the model if it has not been loaded.
    pub(super) fn acquire_models(&mut self, graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>, part_id: PartID) -> &HashMap<String, Rc<Model>> {
        if self.models.contains_key(&part_id) {
            self.models.get(&part_id).unwrap()
        } else {
            let models = self.load_models(graphics, low_poly_shader, part_id);
            self.models.insert(part_id, models);
            self.models.get(&part_id).unwrap()
        }
    }

    /// Actually create the buffers models
    fn load_models(&self, graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>, part_id: PartID) -> HashMap<String, Rc<Model>> {
        let mut output = HashMap::new();
        for (key, value) in self.load_model_data(part_id).into_iter() {
            output.insert(key, Rc::new(match value {
                Mesh::ModelMesh(vertices, indices) => Model::new(graphics, low_poly_shader, VertexType::Specified(vertices, indices), vec![None]).unwrap(),
                Mesh::BumpedMesh(vertices, indices, normal_bytes, _normal_map_strength) => {
                    let normal_map = Input::new_texture(graphics, TextureType::Transparency(&normal_bytes));
                    Model::new(graphics, low_poly_shader, VertexType::Specified(vertices, indices), vec![Some(normal_map)]).unwrap()
                },
                Mesh::ColliderMesh(..) => continue,
            }));
        }
        output
    }

    pub(super) fn load_colliders(&self, part_id: PartID) -> Vec<Collider> {
        let mut output = Vec::new();
        for value in self.load_model_data(part_id).into_values() {
            if let Mesh::ColliderMesh(vertices) = value {
                output.push(Collider::polyhedron(vertices.into_iter().map(
                    |a| Vector3::new(a[0] as f64, a[1] as f64, a[2] as f64)
                ).collect()));
            }
        }
        output
    }

    /// Loads the Mesh model data into a hash map, indexed by path?
    fn load_model_data(&self, part_id: PartID) -> HashMap<String, Mesh> {
        // Look for part id in the paths dict
        match self.paths.get(&part_id) {
            Some(_) => unimplemented!("Loading models from a file is not implemented"),
            None => match part_id { // Assume the data was compiled
                compiled::enterprise::KESTREL => bincode::deserialize(include_model!("../../assets/enterprise/kestrel")).unwrap(),
                compiled::test::CUBE => bincode::deserialize(include_model!("../../assets/test/cube")).unwrap(),
                _ => match MakeID::from(part_id) {
                    compiled::enterprise::MAKE => bincode::deserialize(include_model!("../../assets/enterprise/accessories")).unwrap(),
                    _ => panic!("Models of {:?} were not compiled", part_id)
                }
            }
        }
    }

    pub(super) fn load_ship_data(&self, id: PartID) -> ShipData {
        // Look for part id in the paths dict
        match self.paths.get(&id) {
            Some(_) => unimplemented!("Loading ship data from a file is not implemented"),
            None => match id { // Assume the data was compiled
                compiled::enterprise::KESTREL => bincode::deserialize(include_bytes!("../../assets/enterprise/kestrel/kestrel.dat")).unwrap(),
                compiled::test::CUBE => bincode::deserialize(include_bytes!("../../assets/test/cube/cube.dat")).unwrap(),
                _ => panic!("Ship {:?} was not compiled.", id)
            }
        }
    }

    pub(super) fn load_part_data(&self, id: PartID) -> PartData {
        match self.paths.get(&id) {
            Some(_) => unimplemented!("Loading ship data from a file is not implemented"),
            None => match id { // Assume the data was compiled
                compiled::enterprise::CHAIR => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/chair.dat")).unwrap(),
                compiled::enterprise::DISH => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/dish.dat")).unwrap(),
                compiled::enterprise::PORT => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/port.dat")).unwrap(),
                compiled::enterprise::RADIATOR => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/radiator.dat")).unwrap(),
                compiled::enterprise::RCS => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/rcs.dat")).unwrap(),
                compiled::enterprise::SOLAR => bincode::deserialize(include_bytes!("../../assets/enterprise/accessories/solar.dat")).unwrap(),
                _ => panic!("Part {:?} was not compiled.", id)
            }
        }
    }
}