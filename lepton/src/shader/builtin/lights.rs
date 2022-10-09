use cgmath::Vector4;

use crate::shader::{ShaderStages, Data};
use crate::input::{Input, InputLevel};
use crate::physics::Object;
use crate::Graphics;

pub const NUM_LIGHTS: usize = 2;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightsData {
    pub light_pos: [Vector4<f32>; NUM_LIGHTS],
    pub light_features: [Vector4<f32>; NUM_LIGHTS],
    pub num_lights: u32,
}
impl Data for LightsData {
    const STAGES: ShaderStages = ShaderStages::FRAGMENT;
    const LEVEL: InputLevel = InputLevel::Shader;
}

pub struct LightFeatures {
    pub diffuse_coeff: f32,
    pub specular_coeff: f32,
    pub shininess: u32,
    pub brightness: f32,
}

impl LightFeatures {
    pub(crate) fn as_vec(&self) -> Vector4<f32> {
        Vector4::new(self.diffuse_coeff, self.specular_coeff, self.shininess as f32, self.brightness)
    }
}

pub struct Lights {
    object_indices: [Option<Object>; NUM_LIGHTS],
    light_pos: [Vector4::<f32>; NUM_LIGHTS],
    light_features: [Vector4<f32>; NUM_LIGHTS],
    pub input: Input,
}

impl Lights {
    pub fn new(graphics: &Graphics) -> Self {
        let input = Input::new_buffer::<LightsData>(graphics);
        Self {
            object_indices: [None; NUM_LIGHTS],
            light_pos: [Vector4::new(0.0, 0.0, 0.0, 0.0); NUM_LIGHTS],
            light_features: [Vector4::new(0.0, 0.0, 0.0, 0.0); NUM_LIGHTS],
            input,
        }
    }

    pub fn illuminate(&mut self, object: Object, features: LightFeatures) {
        let mut index = None;
        for (i, val) in self.object_indices.iter().enumerate() {
            if val.is_none() {
                self.object_indices[i] = Some(object);
                index = Some(i);
                break;
            }
        }
        let index = match index {
            Some(i) => i,
            None => panic!("There are too many lights in the scene.")
        };
        self.light_features[index] = features.as_vec();
    }

    pub fn unilluminate(&mut self, object: Object) {
        for (i, val) in self.object_indices.iter().enumerate() {
            if let Some(o) = val {
                if *o == object {
                    self.object_indices[i] = None;
                    break;
                }
            }
        }
    }

    pub fn update_input(&mut self, graphics: &Graphics, buffer_index: usize) {
        let mut num_lights = 0;
        for (index, val) in self.object_indices.iter().enumerate() {
            if let Some(o) = val {
                num_lights += 1;
                let pos = match graphics.get_pos(o) {
                    Some(p) => p,
                    None => continue,
                };
                self.light_pos[index] = Vector4::new(pos.x, pos.y, pos.z, 1.0);
            }
        }
        let data = LightsData {
            light_pos: self.light_pos,
            light_features: self.light_features,
            num_lights,
        };
        self.input.update(data, buffer_index);
    }
}