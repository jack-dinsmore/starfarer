use lepton::load_font;
pub extern crate lepton;

macro_rules! format_ship {
    ( $head_name:literal ) => {
        println!("Making ship {}", $head_name);
        let (vertices, indices) = $crate::lepton::tools::load_obj(&std::path::Path::new(&format!("{}.obj", $head_name))).unwrap();
        let texture_data = $crate::lepton::tools::read_as_bytes(&std::path::Path::new(&format!("{}.png", $head_name)));
        let info_data = $crate::lepton::tools::read_as_bytes(&std::path::Path::new(&format!("{}.dat", $head_name)));

        let info_size = info_data.len() as u64;
        let texture_size = texture_data.len() as u64;
        let num_indices = indices.len() as u64;
        let intro_bytes = [info_size, texture_size, num_indices];
        let vertices_bytes = vertices.iter().map(|v| { $crate::lepton::tools::struct_as_bytes(v)}).collect::<Vec<&[u8]>>().concat();
        let indices_bytes = indices.iter().map(|i| { $crate::lepton::tools::struct_as_bytes(i)}).collect::<Vec<&[u8]>>().concat();


        let out_data = [
            $crate::lepton::tools::struct_as_bytes(&intro_bytes),
            &info_data[..],
            &texture_data[..], 
            &vertices_bytes[..],
            &indices_bytes[..],
        ].concat();

        std::fs::write(&std::path::Path::new(&format!("{}.sfr", $head_name)), out_data).unwrap();
    }
}

fn main() {
    load_font!("assets/fonts/NunitoSans/NunitoSans-Bold.ttf", "assets/fonts/rendered", 24);
    format_ship!("assets/endeavour/accessories/port");
}