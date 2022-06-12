use cgmath::{Vector3, Vector4, Quaternion, Matrix4};

use crate::shader;


pub struct Object {
    pub pos: Vector3<f64>,
    pub orientation: Quaternion<f64>,
    pub(crate) light_index: Option<usize>,
    pub(crate) push_constants: shader::PushConstants
}

impl Object {
    pub fn new(pos: Vector3<f64>, orientation: Quaternion<f64>) -> Self {
        Object {
            pos,
            orientation,
            light_index: None,
            push_constants: shader::PushConstants {
                model: Matrix4::from_scale(1.0),
            }
        }
    }

    pub fn update_input(&mut self) {
        //// Add orientation matrix. Also maybe don't compute data every frame
<<<<<<< Updated upstream
        let data = shader::PushConstants {
            model: Matrix4::from_translation(self.pos.cast().unwrap()),
        };
=======
        shader::PushConstants {
            model: Matrix4::from_translation(self.pos.cast().unwrap()),
        };//.push_constants();
        //// Make this an adjustment to push constants instead.
>>>>>>> Stashed changes
    }

    pub fn update_light(&self, lights: &mut shader::Lights, features: Option<shader::LightFeatures>) {
        if let Some(i) = self.light_index {
            lights.light_pos[i] = Vector4::new(self.pos.x as f32, self.pos.y as f32, self.pos.z as f32, 1.0);
            if let Some(f) = features {
                lights.light_features[i] = f.as_vec();
            }
        }
    }
}