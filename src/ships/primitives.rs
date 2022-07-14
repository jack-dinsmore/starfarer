use serde::{Serialize, Deserialize};
use cgmath::{Vector3, Quaternion, Matrix3};

mod fakes {
    use cgmath::{Vector3, Matrix3, Quaternion};
    use serde::{Serialize, Deserialize};
    
    #[derive(Serialize, Deserialize)]
    #[serde(remote = "Vector3::<f32>")]
    pub struct FakeVector {
        x: f32,
        y: f32,
        z: f32,
    }
    #[derive(Serialize, Deserialize)]
    #[serde(remote = "Matrix3::<f32>")]
    pub struct FakeMatrix {
        #[serde(with = "FakeVector")]
        x: Vector3<f32>,
        #[serde(with = "FakeVector")]
        y: Vector3<f32>,
        #[serde(with = "FakeVector")]
        z: Vector3<f32>,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "Quaternion::<f32>")]
    pub struct FakeQuaternion {
        #[serde(with = "FakeVector")]
        v: Vector3<f32>,
        s: f32,
    }
}
pub use fakes::*;

pub mod compiled {
    pub mod enterprise {
        use super::super::{MakeID, PartID};
        pub const MAKE: MakeID = MakeID{ make: 0 };

        pub const KESTREL: PartID = MAKE.part(0);
        
        pub const CHAIR: PartID = MAKE.part(1);
        pub const DISH: PartID = MAKE.part(2);
        pub const PORT: PartID = MAKE.part(3);
        pub const RADIATOR: PartID = MAKE.part(4);
        pub const RCS: PartID = MAKE.part(5);
        pub const SOLAR: PartID = MAKE.part(6);
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MakeID {
    make: u32,
}
impl From<PartID> for MakeID {
    fn from(m: PartID) -> Self {
        Self { make: m.make }
    }
}
impl MakeID {
    pub const fn part(&self, part: u32) -> PartID {
        PartID { make: self.make, part }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PartID {
    make: u32,
    part: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ShipData {
    pub id: PartID,
    #[serde(with = "FakeVector")]
    pub center_of_mass: Vector3<f32>,
    #[serde(with = "FakeMatrix")]
    pub moment_of_inertia: Matrix3<f32>,
    pub attachments: Vec<PartInstance>,
}

#[derive(Serialize, Deserialize)]
pub struct PartData {
    pub object_name: String,
    pub functions: Vec<Function>,
}

#[derive(Serialize, Deserialize)]
pub struct PartInstance {
    pub id: PartID,
    #[serde(with = "FakeQuaternion")]
    pub orientation: Quaternion::<f32>,
    #[serde(with = "FakeVector")]
    pub position: Vector3<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct Function {
    pub actions: Vec<Action>
}
#[derive(Serialize, Deserialize)]
pub enum Action {
    Execute(PartID, usize)
}

