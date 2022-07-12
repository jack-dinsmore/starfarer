use serde::{Serialize, Deserialize};
use cgmath::{Vector3, Quaternion};
use std::collections::HashMap;
use super::fakes::*;
use super::bytecode::{Function, FunctionID};

#[derive(Serialize, Deserialize)]
pub struct PartInfo {
    #[serde(with = "FakeQuaternion")]
    pub orientation: Quaternion::<f32>,
    #[serde(with = "FakeVector")]
    pub position: Vector3<f32>,
    pub functions: HashMap<FunctionID, Function>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PartID {
    Global { make_id: u32, model_id: u32 },
    Local { model_id: u32 },
}
impl PartID {
    pub const fn global(make_id: u32, model_id: u32) -> Self {
        PartID::Global { make_id, model_id }
    }
    pub const fn local(make_id: u32, model_id: u32) -> Self {
        PartID::Global { make_id, model_id }
    }
    pub const fn append(&self, other: PartID) -> Self {
        if let Self::Global{ make_id, .. } = self {
            return Self::Global {
                make_id: *make_id,
                model_id: match other {
                    Self::Global {model_id, ..} => model_id,
                    Self::Local {model_id} => model_id,
                }
            }
        } else {
            panic!("The append function can only be called from a global ID.")
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