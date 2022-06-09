use cgmath::Vector4;

use crate::shader;

pub struct LightFeatures {
    pub diffuse_coeff: f32,
    pub specular_coeff: f32,
    pub shininess: u32,
}

impl LightFeatures {
    pub(crate) fn as_vec(&self) -> Vector4<f32> {
        Vector4::new(self.diffuse_coeff, self.specular_coeff, self.shininess as f32, 1.0)
    }
}

pub struct Lights {
    index_state: [bool; shader::builtin::NUM_LIGHTS],
    pub(crate) light_pos: [Vector4::<f32>; shader::builtin::NUM_LIGHTS],
    pub(crate) light_features: [Vector4<f32>; shader::builtin::NUM_LIGHTS],
}

impl Lights {
    pub fn new() -> Self {
        Self {
            index_state: [false; shader::builtin::NUM_LIGHTS],
            light_pos: [Vector4::new(0.0, 0.0, 0.0, 0.0); shader::builtin::NUM_LIGHTS],
            light_features: [Vector4::new(0.0, 0.0, 0.0, 0.0); shader::builtin::NUM_LIGHTS],
        }
    }

    pub fn illuminate(&mut self, object: &mut shader::Object, features: LightFeatures) {
        let index = match object.light_index{
            Some(i) => i,
            None => self.pop_index()
        };
        self.light_features[index] = features.as_vec();
        self.light_pos[index] = Vector4::new(object.pos.x as f32, object.pos.y as f32, object.pos.z as f32, 1.0);
        object.light_index = Some(index);
    }

    pub fn unilluminate(&mut self, object: &mut shader::Object) {
        if let Some(i) = object.light_index {
            self.push_index(i);
            object.light_index = None;
        }
    }

    pub fn update_input(&mut self, buffer_index: usize) {
        let mut light_pos = [Vector4::new(0.0, 0.0, 0.0, 0.0); shader::builtin::NUM_LIGHTS];
        let mut light_features = [Vector4::new(0.0, 0.0, 0.0, 0.0); shader::builtin::NUM_LIGHTS];

        let mut num_lights = 0;
        for (index, state) in self.index_state.iter().enumerate() {
            if *state {
                light_pos[num_lights] = self.light_pos[index];
                light_features[num_lights] = self.light_features[index];
                num_lights += 1;
            }
        }

        let data = shader::builtin::LightsData {
            light_pos,
            light_features,
            num_lights: num_lights as u32,
        };
        shader::InputType::Lights.get_input().update(data, buffer_index);
    }

    fn pop_index(&mut self) -> usize {
        for (index, state) in self.index_state.iter().enumerate() {
            if !state {
                self.index_state[index] = true;
                return index;
            }
        }
        panic!("Too many lights have been illuminated");
    }

    fn push_index(&mut self, index: usize) {
        self.index_state[index] = false;
    }
}