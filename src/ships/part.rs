use lepton::prelude::*;
use serde::{Serialize, Deserialize};
use cgmath::{Vector3, Matrix4, Matrix3, Quaternion};
use super::primitives::*;
use std::rc::Rc;

pub struct PartState {
    pub model: Option<Rc<Model>>,
    pub matrix: Matrix4<f32>,
}
impl PartState {
    pub fn from_instance(instance: PartInstance, model: Rc<Model> ) -> Self {
        let rotation = Matrix4::from(Matrix3::from(instance.orientation.cast().unwrap()));
        Self {
            model: Some(model),
            matrix: Matrix4::from_translation(instance.position) * rotation,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Element {
    element_type: ElementType,
    #[serde(with = "FakeVector")]
    pos: Vector3<f32>,
    #[serde(with = "FakeQuaternion")]
    orientation: Quaternion<f32>,
}

#[derive(Serialize, Deserialize)]
enum ElementType {
    Display,
    Button,
    Switch,
    DigitalDial,
    AnalogDial,
    Lever,
    Slider,
    Wheel,
}