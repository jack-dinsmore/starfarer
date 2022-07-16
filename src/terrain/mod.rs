mod triangulation;
mod triangle;

use std::collections::HashMap;
use lepton::prelude::*;
use cgmath::Vector3;
use triangle::*;

const NUM_OCTAVES: u8 = 8;

const HIGH_DEGREE: u8 = 0;

pub struct Terrain {
    settings: TerrainSettings,
    seed: u32,
    power: f32,
    radius: f64,

    object: Object,
    models: HashMap<MapID, (u8, Model)>,
}

impl Terrain {
    pub fn new(seed: u32, radius: f64, object_manager: &mut ObjectManager) -> Self {
        //// Eventually choose these as functions of radius
        let face_subdivision = 1;
        let map_subdivision = 5;
        let height_subdivision = 5;
        let power = 1.0;
        let height = 0.1;
        Self {
            seed,
            power,
            radius,
            settings: TerrainSettings {
                face_subdivision,
                map_subdivision,
                height_subdivision,
                height,
            },

            object: object_manager.get_object(),
            models: HashMap::new(),
        }
    }

    pub fn update(&self, graphics: &Graphics, shader: &Shader<builtin::LPSignature>, position: Vector3<f64>) {
        // Load extras
        self.load_map(graphics, shader, Triangle::get_id(position), HIGH_DEGREE);
    }

    pub fn render<'a>(&'a self, tasks: &mut Vec<RenderTask<'a>>){
        // Render
        for (_, (_, model)) in &self.models {
            tasks.push(RenderTask::DrawModelWithObject(self.object, model));
        }
    }

    fn load_map(&mut self, graphics: &Graphics, shader: &Shader<builtin::LPSignature>, id: MapID, degree: u8) {
        let new_model = match self.models.remove(&id) {
            Some(data) => match data.0 {
                degree => return,
                _ => self.models.insert(id, (degree, self.load_from_old(graphics, shader, id, degree, data.0, data.1)))
            },
            None => self.models.insert(id, (degree, self.load_new(graphics, shader, id, degree)))
        };
    }

    fn load_from_old(&self, graphics: &Graphics, shader: &Shader<builtin::LPSignature>, id: MapID,
        new_degree: u8, old_degree: u8, old_model: Model) -> Model {

        return self.load_new(graphics, shader, id, new_degree);
        //// Actually implement the function instead
        //unimplemented!();
    }

    fn load_new(&self, graphics: &Graphics, shader: &Shader<builtin::LPSignature>, id: MapID, degree: u8) -> Model {
        let (vertices, indices) = Triangle::new(id, degree, &self.settings, |pos| {Self::height_fn(pos)}).load_new();
        Model::new(graphics, shader, VertexType::Specified(vertices, indices), TextureType::None).unwrap()
    }

    fn height_fn(pos: [f64; 3]) -> f64 {
        unimplemented!()
    }
}