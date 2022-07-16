mod triangulation;
mod square;

use std::collections::HashMap;
use lepton::prelude::*;
use cgmath::{Vector3, Matrix3, Quaternion, Zero, Matrix};
use noise::{OpenSimplex, Seedable, NoiseFn};
use square::*;

const NUM_OCTAVES: u8 = 8;

const HIGH_DEGREE: u8 = 0; // Power of 2
const LOW_DEGREE: u8 = 2;

enum LoadState {
    High(Model),
    Low(Model),
    Unloaded,
}

pub struct Planet {
    settings: PlanetSettings,
    noise_map: OpenSimplex,
    power: f32,

    object: Object,
    models: Vec<(MapID, LoadState)>,
    last_id: Option<MapID>,
}

impl Planet {
    pub fn new(seed: u32, radius: f64, object_manager: &mut ObjectManager) -> Self {
        //// Eventually choose these as functions of radius
        let face_subdivision = 3;
        let map_subdivision = 5;
        let height_subdivision = 5;
        let power = 1.0;
        let height = 0.1;
        let mut models = Vec::with_capacity((face_subdivision * face_subdivision) as usize * 6);
        for face in 0..6 {
            for map_row in 0..(face_subdivision as u8) {
                for map_col in 0..(face_subdivision as u8) {
                    models.push((MapID { face, map_row, map_col }, LoadState::Unloaded));
                }
            }
        }
        let noise_map = OpenSimplex::new().set_seed(seed);
        Self {
            noise_map,
            power,
            settings: PlanetSettings {
                face_subdivision,
                map_subdivision,
                height_subdivision,
                height,
                radius,
            },

            object: object_manager.get_object(),
            models,
            last_id: None,
        }
    }

    pub fn update(&mut self, graphics: &Graphics, shader: &Shader<builtin::LPSignature>, position: &Vector3<f32>) {
        // Load extras
        let (planet_pos, planet_rot) = match graphics.get_pos_and_rot(&self.object) {
            Some(data) => data,
            None => (Vector3::zero(), Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)),
        };
        let my_id = square::get_id((planet_rot.transpose() * (position - planet_pos)).cast().unwrap(), self.settings.face_subdivision);
        if let Some(id) = self.last_id {
            if id == my_id {
                return; // No need to load
            }
        }
        self.last_id = Some(my_id);

        for (id, model) in self.models.iter_mut() {
            let dist = MapID::dist(my_id, *id, self.settings.face_subdivision as u8);
            if dist == 0 { // Load high
                match model {
                    LoadState::High(_) => (),
                    LoadState::Low(m) => *model = LoadState::High(
                        Model::new(graphics, shader, Square::new(
                            *id, HIGH_DEGREE, &self.settings,|pos| {
                                Self::height_fn(pos, self.settings.radius, self.noise_map, self.power)
                            }).load_from_old(m, LOW_DEGREE), TextureType::None
                        ).unwrap()
                    ),
                    LoadState::Unloaded => *model = LoadState::High(
                        Model::new(graphics, shader, Square::new(
                            *id, HIGH_DEGREE, &self.settings, |pos| {
                                Self::height_fn(pos, self.settings.radius, self.noise_map, self.power)
                            }).load_new(), TextureType::None
                        ).unwrap()
                    ),
                }
            } else if dist == 1 {
                match model {
                    LoadState::High(m) => *model = LoadState::Low(
                        Model::new(graphics, shader, Square::new(
                            *id, LOW_DEGREE, &self.settings,|pos| {
                                Self::height_fn(pos, self.settings.radius, self.noise_map, self.power)
                            }).load_from_old(m, HIGH_DEGREE), TextureType::None
                        ).unwrap()
                    ),
                    LoadState::Low(_) => (),
                    LoadState::Unloaded => *model = LoadState::Low(
                        Model::new(graphics, shader, Square::new(
                            *id, LOW_DEGREE, &self.settings,|pos| {
                                Self::height_fn(pos, self.settings.radius, self.noise_map, self.power)
                            }).load_new(), TextureType::None
                        ).unwrap()
                    ),
                }
            } else {// Unloaded
                *model = LoadState::Unloaded
            };
        }
    }

    pub fn render<'a>(&'a self, tasks: &mut Vec<RenderTask<'a>>){
        // Render
        for (_, model) in &self.models {
            match model {
                LoadState::Unloaded => (),
                LoadState::High(m) => tasks.push(RenderTask::DrawModelWithObject(self.object, m)),
                LoadState::Low(m) => tasks.push(RenderTask::DrawModelWithObject(self.object, m)),
            }
        }
    }

    pub fn init_rigid_body(&self, map: &mut HashMap<Object, RigidBody>) {
        map.insert(self.object, RigidBody::new(
            Vector3::new(2000.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.5)
        ).motivate(1.0, Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)));
    }
}

impl Planet {
    fn height_fn(pos: [f64; 3], radius: f64, noise_map: OpenSimplex, power: f32) -> f64 {
        let mut val = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]) / (radius * radius) - 1.0;
        //val += noise_map.get(pos);
        val
    }
}