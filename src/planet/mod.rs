#![allow(dead_code)]

mod triangulation;
mod square;

use rustc_hash::FxHashMap;
use lepton::prelude::*;
use cgmath::{Vector3, Matrix3, Quaternion, Zero, Matrix, InnerSpace};
use noise::{OpenSimplex, Seedable, NoiseFn};
use square::*;
use super::threadpool::ThreadPool;
use std::sync::mpsc::Receiver;

const NUM_OCTAVES: u8 = 5;
const UPDATE_PERIOD: u8 = 8;

#[derive(Copy, Clone)]
pub enum LoadDegree {
    High = 0,
    Low = 1,
}

enum LoadState {
    High(Model),
    Low(Model),
    Unloaded,
}

#[derive(Debug, PartialEq)]
struct LoadConfig {
    low_dist: i32,
    high_dist: i32,
}

pub struct Planet {
    settings: PlanetSettings,
    noise_map: OpenSimplex,
    power: f64,
    load_configs: Vec<(f32, LoadConfig)>,
    update_frame: u8,

    object: Object,
    models: Vec<(MapID, LoadState)>,
    last_state: Option<(MapID, usize)>,
    model_switches: Vec<(usize, LoadDegree, Receiver<(Vec<vertex::VertexLP>, Vec<u32>)>)>,
}

impl Planet {
    pub fn new(seed: u32, radius: f64, object_manager: &mut ObjectManager) -> Self {
        //// Eventually choose these as functions of radius
        let face_subdivision = 4;
        let map_subdivision = 16;
        let request_height = 0.2;

        // Calculate height
        let triangle_length = std::f64::consts::PI / 2.0 / face_subdivision as f64 / map_subdivision as f64;
        let request_divisions = request_height / triangle_length;
        let height_subdivision = closest_multiple_of(1 << (LoadDegree::Low as u8), request_divisions);
        let height = triangle_length * height_subdivision as f64;

        println!("Planet height: {} with division {}", height, height_subdivision);

        let power = 0.8;
        let mut models = Vec::with_capacity((face_subdivision * face_subdivision) as usize * 6);
        for face in 0..6 {
            for map_row in 0..(face_subdivision as u8) {
                for map_col in 0..(face_subdivision as u8) {
                    models.push((MapID { face, map_row, map_col }, LoadState::Unloaded));
                }
            }
        }
        let noise_map = OpenSimplex::new().set_seed(seed);

        let load_configs = vec![// Sorted from high to low
            (1.4, LoadConfig { low_dist: (face_subdivision as f64 * 1.5) as i32, high_dist: -1 } ),
            (1.0, LoadConfig { low_dist: 2, high_dist: 0 } ),
        ];

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
            load_configs,
            update_frame: 0,

            object: object_manager.get_object(),
            models,
            last_state: None,
            model_switches: Vec::new(),
        }
    }

    pub fn update(&mut self, graphics: &Graphics, shader: &Shader<builtin::LPSignature>,
        threadpool: &ThreadPool, position: &Vector3<f32>) {

        // Manage switches
        for switch in self.model_switches.iter_mut() {
            if let Ok(data) = switch.2.try_recv() {
                let new_model = Model::new(graphics, shader, VertexType::Specified(data.0, data.1), TextureType::None).unwrap();
                self.models[switch.0].1 = match switch.1 {
                    LoadDegree::High => LoadState::High(new_model),
                    LoadDegree::Low => LoadState::Low(new_model),
                };
            }
        }

        self.update_frame = (self.update_frame + 1) % UPDATE_PERIOD;
        if self.update_frame != 0 { return; }

        // Get current position
        let (planet_pos, planet_rot) = match graphics.get_pos_and_rot(&self.object) {
            Some(data) => data,
            None => (Vector3::zero(), Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)),
        };
        let my_id = square::get_id((planet_rot.transpose() * (position - planet_pos)).cast().unwrap(), self.settings.face_subdivision);

        // Get load configuration
        let height_ratio = (position - planet_pos).magnitude() / self.settings.radius as f32;
        let mut config_iter = self.load_configs.iter().enumerate();
        let (my_config_index, my_load_config) = loop {
            match config_iter.next() {
                Some((i, (threshold, config))) => {
                    if *threshold < height_ratio {
                        break (i, config);
                    }
                },
                None => break (self.load_configs.len() - 1, &self.load_configs.last().unwrap().1),
            }
        };

        if let Some((id, config_index)) = self.last_state {
            if id == my_id && config_index == my_config_index {
                return; // No need to load
            }
        }
        self.last_state = Some((my_id, my_config_index));

        
        // Load new maps
        for (index, (id, model)) in self.models.iter_mut().enumerate() {
            let dist = MapID::dist(my_id, *id, self.settings.face_subdivision as u8);
            if dist <= my_load_config.high_dist { // Load high
                if let LoadState::High(_) = model {}
                else {
                    let settings = self.settings;
                    let noise_map = self.noise_map;
                    let power = self.power;
                    let id_val = *id;
                    self.model_switches.push((index, LoadDegree::High, threadpool.execute(move || {
                        Square::new(id_val, LoadDegree::High, settings, |pos| {
                            Self::height_fn(pos, noise_map, power)
                        }).load_new()
                    })));
                }
            } else if dist <= my_load_config.low_dist {// Load low
                if let LoadState::Low(_) = model {}
                else {
                    let settings = self.settings;
                    let noise_map = self.noise_map;
                    let power = self.power;
                    let id_val = *id;
                    self.model_switches.push((index, LoadDegree::Low, threadpool.execute(move || {
                        Square::new(id_val, LoadDegree::Low, settings, |pos| {
                            Self::height_fn(pos, noise_map, power)
                        }).load_new()
                    })));
                }
            } else {// Unload
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

    pub fn init_rigid_body(&self, map: &mut FxHashMap<Object, RigidBody>) {
        map.insert(self.object, RigidBody::new(
            Vector3::new(2000.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.5)
        ).motivate(1.0, Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)));
    }
}

impl Planet {
    fn height_fn(pos: [f64; 3], noise_map: OpenSimplex, power: f64) -> f64 {
        const TERRACES: f64 = 8.0;
        let mut val = 0.0;
        for oct in 0..NUM_OCTAVES {
            let freq = 1 << oct;
            val += (0.5 + 5.0 * noise_map.get([pos[0] * freq as f64, pos[1] * freq as f64, pos[2] * freq as f64]) / (freq as f64)).clamp(0.0, 1.0);
        }
        val /= NUM_OCTAVES as f64;
        0.5 * (((val.powi(3)-0.15) * TERRACES).round() / TERRACES) + pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2] - 1.0
    }
}

fn closest_multiple_of(multiple: u32, number: f64) -> u32 {
    let ratio = number / multiple as f64;
    if (ratio - (ratio as u32) as f64) < 0.5 {
        ratio as u32 * 4
    } else {
        (ratio as u32 + 1) * 4
    }
}