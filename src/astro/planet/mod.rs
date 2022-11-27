#![allow(dead_code)]

mod triangulation;
mod square;
pub(super) mod primitives;

use lepton::prelude::*;
use cgmath::{Vector3, Matrix3, Zero, Matrix, InnerSpace};
use noise::{OpenSimplex, NoiseFn};
use square::*;
use crate::threadpool::ThreadPool;
use std::sync::mpsc::Receiver;
use primitives::*;

const NUM_OCTAVES: u8 = 7;
const UPDATE_PERIOD: u8 = 8;
const AMPLITUDE: [f64; NUM_OCTAVES as usize] = [
    2.0, 1.5/2.0, 1.5/4.0, 1.0/8.0, 1.0/16.0, 1.0/32.0, 1.0/64.0
];
pub(super) const SCALE_TO_HEIGHT_RATIO: f64 = 1.2;

enum LoadState {
    High(Model),
    Low(Model),
    Unloaded,
}

#[derive(Clone, Copy)]
pub struct Atmosphere {
    pub base_pressure: f32, // Atmospheres
    pub scale_height: f32,
    pub min_alpha: f32,
}

#[derive(Debug, PartialEq)]
struct LoadConfig {
    low_dist: i32,
    high_dist: i32,
}

pub(super) struct Planet {
    pub settings: PlanetSettings,
    load_configs: Vec<(f32, LoadConfig)>,
    update_frame: u8,

    pub object: Object,
    models: Vec<(MapID, LoadState)>,
    last_state: Option<(MapID, usize)>,
    model_switches: Vec<(usize, LoadDegree, Receiver<(Vec<vertex::VertexLP>, Vec<u32>)>)>,

    pub atmosphere: Option<Atmosphere>,
}

impl Planet {
    pub(super) fn new(settings: PlanetSettings, object: Object) -> Self {
        let atmosphere = Atmosphere::new(1.0, 20.0);

        let mut models = Vec::with_capacity((settings.face_subdivision * settings.face_subdivision) as usize * 6);
        for face in 0..6 {
            for map_row in 0..(settings.face_subdivision as u8) {
                for map_col in 0..(settings.face_subdivision as u8) {
                    models.push((MapID { face, map_row, map_col }, LoadState::Unloaded));
                }
            }
        }

        let load_configs = vec![// Sorted from high to low
            (1.4, LoadConfig { low_dist: (settings.face_subdivision as f64 * 1.5) as i32, high_dist: -1 } ),
            (1.0, LoadConfig { low_dist: 2, high_dist: 1 } ),
        ];

        Self {
            settings,
            load_configs,
            update_frame: 0,

            object,
            models,
            last_state: None,
            model_switches: Vec::new(),
            atmosphere: Some(atmosphere),
        }
    }

    pub fn update(&mut self, graphics: &Graphics, shader: &Shader<builtin::LPSignature>,
        threadpool: &ThreadPool, position: &Vector3<f32>) {

        // Manage switches
        for switch in self.model_switches.iter_mut() {
            if let Ok(data) = switch.2.try_recv() {
                let new_model = Model::new(graphics, shader, VertexType::Specified(data.0, data.1), vec![None]).unwrap();
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
                    let noise_map = self.settings.noise_map;
                    let id_val = *id;
                    let scale = self.settings.height * SCALE_TO_HEIGHT_RATIO;
                    self.model_switches.push((index, LoadDegree::High, threadpool.execute(move || {
                        Square::new(id_val, LoadDegree::High, settings, |pos| {
                            Self::value_fn(pos, noise_map, settings.spikiness, scale)
                        }).load_new()
                    })));
                }
            } else if dist <= my_load_config.low_dist {// Load low
                if let LoadState::Low(_) = model {}
                else {
                    let settings = self.settings;
                    let noise_map = self.settings.noise_map;
                    let id_val = *id;
                    let scale = self.settings.height * SCALE_TO_HEIGHT_RATIO;
                    self.model_switches.push((index, LoadDegree::Low, threadpool.execute(move || {
                        Square::new(id_val, LoadDegree::Low, settings, |pos| {
                            Self::value_fn(pos, noise_map, settings.spikiness, scale)
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
}

impl Planet {
    /// Returns the height in radial units of point n_pos above radius one.
    fn height_fn(n_pos: Vector3<f64>, noise_map: OpenSimplex, power: i32, scale: f64) -> f64 {
        // Require n_pos to be normalized
        let mut val = 0.0;
        for oct in 0..NUM_OCTAVES {
            let freq = (1 << oct) as f64;
            val += (0.5 + 5.0 * noise_map.get([n_pos.x * freq, n_pos.y * freq, n_pos.z * freq]) * AMPLITUDE[oct as usize]).clamp(0.0, 1.0);
        }
        scale * ((val / NUM_OCTAVES as f64).powi(power)-1.0 / (power as f64+ 1.0))
    }

    /// Returns the value of a specific location, with zero being the surface
    pub(super) fn value_fn(pos: Vector3<f64>, noise_map: OpenSimplex, power: i32, scale: f64) -> f64 {
        let mag = pos.magnitude();
        let n_pos = pos / mag;
        return Self::height_fn(n_pos, noise_map, power, scale) + (mag - 1.0)
    }
}

impl Atmosphere {
    fn new(base_pressure: f32, scale_height: f32) -> Self {
        Self {
            base_pressure,
            scale_height,
            min_alpha: Self::get_min_alpha(base_pressure),
        }
    }

    fn get_min_alpha(base_pressure: f32) -> f32 {
        base_pressure
    }
}