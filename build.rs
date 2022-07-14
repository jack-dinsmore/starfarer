use std::path::Path;
use std::fs;

include!("src/ships/primitives.rs");

fn enterprise() {
    const ROOT_PATH: &'static str = "assets/enterprise/";
    
    fs::write(Path::new(&format!("{}/kestrel/kestrel.dat", ROOT_PATH)), bincode::serialize(&ShipData {
        id: compiled::enterprise::KESTREL,
        center_of_mass: Vector3::new(0.0, 0.0, 0.0),
        moment_of_inertia: Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 1.0,
        ),
        attachments: vec![
            PartInstance {
                id: compiled::enterprise::PORT,
                orientation: Quaternion::new(1.0, 0.0, 0.0 ,0.0),
                position: Vector3::new(0.0, 0.0, 3.0),
            }
        ]
    }).unwrap()).unwrap();
    
    fs::write(Path::new(&format!("{}/accessories/chair.dat", ROOT_PATH)), bincode::serialize(&PartData {
        object_name: "chair_Cube.006".to_owned(),
        functions: Vec::new(),
    }).unwrap()).unwrap();
    
    fs::write(Path::new(&format!("{}/accessories/dish.dat", ROOT_PATH)), bincode::serialize(&PartData {
        object_name: "dish_Cube.007".to_owned(),
        functions: Vec::new(),
    }).unwrap()).unwrap();
    
    fs::write(Path::new(&format!("{}/accessories/port.dat", ROOT_PATH)), bincode::serialize(&PartData {
        object_name: "port_Cube.005".to_owned(),
        functions: Vec::new(),
    }).unwrap()).unwrap();
    
    fs::write(Path::new(&format!("{}/accessories/radiator.dat", ROOT_PATH)), bincode::serialize(&PartData {
        object_name: "radiator_Cube.009".to_owned(),
        functions: Vec::new(),
    }).unwrap()).unwrap();
    
    fs::write(Path::new(&format!("{}/accessories/rcs.dat", ROOT_PATH)), bincode::serialize(&PartData {
        object_name: "rcs_Cube.002".to_owned(),
        functions: Vec::new(),
    }).unwrap()).unwrap();
    
    fs::write(Path::new(&format!("{}/accessories/solar.dat", ROOT_PATH)), bincode::serialize(&PartData {
        object_name: "solar_Cube.008".to_owned(),
        functions: Vec::new(),
    }).unwrap()).unwrap();
}

fn main() {
    enterprise();
}