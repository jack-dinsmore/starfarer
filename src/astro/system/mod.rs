use cgmath::{Vector3, Quaternion};
use rand::{SeedableRng, rngs::SmallRng};
use lepton::prelude::*;
use rustc_hash::FxHashMap;
use crate::threadpool::ThreadPool;
use noise::{OpenSimplex, Seedable};

use super::planet::{Planet, Atmosphere, SCALE_TO_HEIGHT_RATIO,
    primitives::{PlanetSettings, ColorScheme, LoadDegree}
};

const LOAD_DISTANCE: f32 = 10_000.0;
const UNLOAD_DISTANCE: f32 = 15_000.0;

fn closest_multiple_of(multiple: u32, number: f64) -> u32 {
    let ratio = number / multiple as f64;
    if (ratio - (ratio as u32) as f64) < 0.5 {
        ratio as u32 * 4
    } else {
        (ratio as u32 + 1) * 4
    }
}

pub struct SolarSystem {
    _seed: [u8; 32],
    loaded_planets: Vec<Option<Planet>>,
    objects: Vec<Object>,
    rigid_bodies: Vec<RigidBody>,
    settings: Vec<PlanetSettings>,
    current_index: Option<usize>,
}

impl SolarSystem {
    pub fn new(seed: [u8; 32], object_manager: &mut ObjectManager, time: f32) -> Self {
        let mut rng = SmallRng::from_seed(seed);
        let num = Self::count_planets(&mut rng);
        let objects = (0..num).map(|_| object_manager.get_object()).collect::<Vec<_>>();
        let settings = (0..num).map(|i| Self::load_setting(&mut rng, i)).collect::<Vec<_>>();
        let rigid_bodies = (0..num).map(|i| Self::load_rigid_body(&mut rng, settings[i], time)).collect::<Vec<_>>();
        let loaded_planets = (0..num).map(|i| Self::initial_load(settings[i], objects[i])).collect::<Vec<_>>();

        Self {
            _seed: seed,
            loaded_planets,
            objects,
            rigid_bodies,
            settings,
            current_index: Some(1),
        }
    }

    /// How many planets are there?
    fn count_planets(_rng: &mut SmallRng) -> usize {
        2
    }

    /// Generate settings for each planet in the solar system
    fn load_setting(_rng: &mut SmallRng, index: usize) -> PlanetSettings {
        let noise_seed = 45678;
        let noise_map = OpenSimplex::new().set_seed(noise_seed);

        if index == 0 {
            let radius = 1000.0;
            let color_scheme = ColorScheme::Single([1.0, 1.0, 1.0]);
    
            PlanetSettings {
                face_subdivision: 4,
                map_subdivision: 64,
                height_subdivision: 1,
                height: 1.0,
                radius,
                color_scheme,
                spikiness: 0,
                noise_seed,
                noise_map,
                is_star: true
            }
        } else {
            let face_subdivision = 4;
            let map_subdivision = 64;
            let request_height = 0.2;
            let spikiness = 3;
            let radius = 1_000.0;
            let color_scheme = ColorScheme::make_double([0.8, 0.7, 0.3], [0.4, 0.3, 0.4], 0.5, 1000.0, 20.0);
    
            // Calculate height
            let triangle_length = std::f64::consts::PI / 2.0 / face_subdivision as f64 / map_subdivision as f64;
            let request_divisions = request_height / triangle_length;
            let height_subdivision = closest_multiple_of(1 << (LoadDegree::Low as u8), request_divisions);
            let height = triangle_length * height_subdivision as f64;
    
            PlanetSettings {
                face_subdivision: 4,
                map_subdivision: 64,
                height_subdivision,
                height,
                radius,
                color_scheme,
                spikiness,
                noise_seed,
                noise_map,
                is_star: false,
            }
        }
    }

    /// Load any planet which should be initially loaded at the conception of the solar system.
    fn initial_load(setting: PlanetSettings, object: Object) -> Option<Planet> {
        if true {
            Some(Planet::new(setting, object))
            // Load the planet
        } else {
            None
        }
    }

    fn load_rigid_body(_rng: &SmallRng, settings: PlanetSettings, time: f32) -> RigidBody {
        let noise_map = settings.noise_map.clone();
        let radius = settings.radius;
        let spikiness = settings.spikiness;
        let scale = settings.height * SCALE_TO_HEIGHT_RATIO;

        let initial_pos = if settings.is_star {
            Vector3::new(0.0, 0.0, 0.0)
        } else {
            Vector3::new(8_990.0, 0.0, 0.0)
        };

        // TODO Eventually, crank up the initial pos to the current time.

        RigidBody::new(
            initial_pos, Vector3::new(0.0, 0.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0)
        )
        // .gravitate(1_000_000.0)
        .collide(vec![
            Collider::planet(Box::new(move |pos| {
                Planet::value_fn(pos / radius, noise_map, spikiness, scale) * radius
            }), (1.0 + settings.height) * settings.radius)
        ], 0.3)
    }
}

impl SolarSystem {
    /// Initialize all the rigid bodies
    pub fn init_rigid_body(&mut self, map: &mut FxHashMap<Object, RigidBody>) {
        for (object, body) in self.objects.iter().zip(self.rigid_bodies.drain(..)) {
            map.insert(*object, body);
        }
    }

    /// Update which planets are loaded and which aren't.
    pub fn update(&mut self, graphics: &Graphics, low_poly_shader: &Shader<builtin::LPSignature>, threadpool: &ThreadPool, player_pos: &Vector3<f32>) {
        for (i, planet) in self.loaded_planets.iter_mut().enumerate() {
            match planet {
                None => {
                    // Check to see if the planet should remain unloaded

                },
                Some(planet) => {
                    // Check to see if the planet should remain loaded
                    planet.update(graphics, low_poly_shader, threadpool, player_pos);
                }
            }
        }
    }

    /// Update which planets are loaded and which aren't.
    pub fn render<'c, 'a: 'c, 'b>(&'a self, tasks: &'b mut Vec<RenderTask<'c>>) {
        for planet in &self.loaded_planets {
            if let Some(planet) = &planet {
                planet.render(tasks);
            } 
        }
    }

    pub fn get_skybox_data(&self) -> Option<(Object, Option<Atmosphere>, f32)> {
        if let Some(index) = self.current_index {
            if let Some(planet) = &self.loaded_planets[index] {
                return Some((
                    planet.object,
                    planet.atmosphere,
                    planet.settings.radius as f32,
                ));
            }
        }
        None
    }
}