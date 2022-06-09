use cgmath::{Vector3, Vector4, Quaternion, Matrix4};

use crate::shader;


pub struct Object {
    pub pos: Vector3<f64>,
    pub orientation: Quaternion<f64>,
    pub(crate) light_index: Option<usize>,
}

impl Object {
    pub fn new(pos: Vector3<f64>, orientation: Quaternion<f64>) -> Self {
        Object {
            pos,
            orientation,
            light_index: None,
        }
    }

    pub fn update_input(&mut self, buffer_index: usize) {
        //// Add orientation matrix. Also maybe don't compute data every frame
        let data = shader::builtin::ObjectData {
            model: Matrix4::from_translation(self.pos.cast().unwrap()),
        };

        shader::InputType::Object.get_input().update(data, buffer_index);
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