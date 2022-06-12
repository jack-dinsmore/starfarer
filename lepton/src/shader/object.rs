use cgmath::{Vector3, Vector4, Quaternion, Matrix4};
use std::rc::Rc;

use crate::shader;
use crate::model::Model;


pub struct Object {
    pub pos: Vector3<f64>,
    pub orientation: Quaternion<f64>,
    pub(crate) light_index: Option<usize>,
    pub(crate) push_constants: shader::PushConstants,
    push_constants_accurate: bool,
    pub(crate) model: Option<Rc<Model>>,
}

impl Object {
    pub fn new(pos: Vector3<f64>, orientation: Quaternion<f64>) -> Self {
        Object {
            pos,
            orientation,
            light_index: None,
            push_constants: shader::PushConstants {
                model: Matrix4::from_scale(1.0),
            },
            model: None,
            push_constants_accurate: false,
        }
    }

    pub fn add_model(&mut self, model: Rc<Model>) {
        self.model = Some(model);
    }

    pub fn set_pos(&mut self, pos: Vector3<f64>) {
        self.pos = pos;
        self.push_constants_accurate = false;
    }

    pub(crate) fn make_push_constants(&mut self) {
        if !self.push_constants_accurate { 
            self.push_constants = shader::PushConstants {
                model: Matrix4::from_translation(self.pos.cast().unwrap()),
            };
            self.push_constants_accurate = true;
        }
    }
    
    pub(crate) fn get_push_constant_bytes(&self) -> &[u8] {
        unsafe { crate::tools::struct_as_bytes(&self.push_constants) }
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