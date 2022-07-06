#[macro_export]
macro_rules! load_obj {
    ( $path_name:expr ) => {
        let path = std::path::Path::new($path_name);
        let model_obj = match $crate::macros::tobj::load_obj(path, &tobj::LoadOptions{single_index: true, ..Default::default()}) {
            Ok(m) => m,
            Err(_) => panic!("Failed to load model object {}", path.display())
        };

        let mut vertices = vec![];
        let mut indices = vec![];

        let (models, _) = model_obj;
        for m in models.iter() {
            let mesh = &m.mesh;

            if mesh.texcoords.len() == 0 {
                panic!("Missing texture coordinates");
            }

            let total_vertices_count = mesh.positions.len() / 3;
            for i in 0..total_vertices_count {
                let vertex = VertexModel {
                    pos: [
                        mesh.positions[i * 3],
                        mesh.positions[i * 3 + 1],
                        mesh.positions[i * 3 + 2],
                    ],
                    normal: [
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    ],
                    tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
                };
                vertices.push(vertex);
            }

            indices = mesh.indices.clone();
        }

        (vertices, indices)
    }
}