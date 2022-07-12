use super::*;

use std::path::Path;
use std::fs;
use std::collections::HashMap;
use cgmath::{Vector3, Matrix3, Quaternion};

mod ids {
    #![allow(dead_code)]
    pub mod astroworks {
        use crate::ships::PartID;
        pub const MAKE_ID: u32 = 0;
        pub const STARLING: PartID = PartID::global(MAKE_ID, 0);
        pub const STARLING_GLASS: PartID = PartID::global(MAKE_ID, 2);
        pub const DOCKING_PORT: PartID = PartID::global(MAKE_ID, 1);
        pub const RADIATOR_OPEN: PartID = PartID::global(MAKE_ID, 2);
        pub const RADIATOR_CLOSED: PartID = PartID::global(MAKE_ID, 2);
        pub const SOLAR_OPEN: PartID = PartID::global(MAKE_ID, 2);
        pub const SOLAR_CLOSED: PartID = PartID::global(MAKE_ID, 2);
        pub const RCS: PartID = PartID::global(MAKE_ID, 2);
    }
}


#[test]
fn astroworks() {
    const ROOT_PATH: &'static str = "assets/astroworks/";
    
    fs::write(Path::new(&format!("{}/starling.starling.dat", ROOT_PATH)), bincode::serialize(&ShipData {
        id: ids::astroworks::STARLING,
        center_of_mass: Vector3::new(0.0, 0.0, 0.0),
        moment_of_inertia: Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        ),
        attachments: HashMap::from([
            (ids::astroworks::DOCKING_PORT, PartInfo {
                orientation: Quaternion::new(1.0, 0.0, 0.0 ,0.0),
                position: Vector3::new(0.0, 0.0, 0.0),
                functions: HashMap::new(),
            }),
        ])
    }).unwrap()).unwrap();
}