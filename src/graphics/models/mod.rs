use glow::*;
use wavefront_obj::{obj};

pub struct Model {
    objects: Vec<Object>,
}

struct Object {
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    ibos: Vec<(NativeBuffer, i32)>
}


impl Object {
    unsafe fn new(gl: &glow::Context, object: obj::Object) -> Object {

        let mut triangle_vertices = Vec::with_capacity(object.vertices.len() * 3);
        for v in object.vertices {
            triangle_vertices.push(v.x);
            triangle_vertices.push(v.y);
            triangle_vertices.push(v.z);
        }

        let triangle_vertices_u8: &[u8] = core::slice::from_raw_parts(
            triangle_vertices.as_ptr() as *const u8,
            triangle_vertices.len() * core::mem::size_of::<f32>(),
        );

        // We construct a buffer and upload the data
        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, triangle_vertices_u8, glow::STATIC_DRAW);
    
        // We now construct a vertex array to describe the format of the input buffer
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

        let mut ibos = Vec::new();
        for geo in object.geometry {
            for shape in geo.shapes {
                match shape.primitive{
                    obj::Primitive::Triangle(..) => (),
                    _ => panic!("Cannot load non-triangle objects"),
                }

                println!("{:?}", shape.smoothing_groups);

                // We construct a buffer and upload the data
                let smoothing_groups_u8: &[u8] = core::slice::from_raw_parts(
                    shape.smoothing_groups.as_ptr() as *const u8,
                    shape.smoothing_groups.len() * core::mem::size_of::<u32>(),
                );

                let ibo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));
                gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, smoothing_groups_u8, glow::STATIC_DRAW);
                ibos.push((ibo, shape.smoothing_groups.len() as i32));
            }
        }

        Object { vao, vbo, ibos }
    }

    fn clean(self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);

            for (ibo, _) in self.ibos {
                gl.delete_buffer(ibo);
            }
        }
    }
}



impl Model {
    pub fn new(gl: &glow::Context, asset_text: &str) -> Model {
        let cube_objs = obj::parse(asset_text).expect("Could not parse cube object");

        let mut objects = Vec::new();
        
        unsafe {
            for object in cube_objs.objects {
                objects.push(Object::new(gl, object));
            }
        }

        Model {objects}
    }

    pub fn draw(&self, gl: &glow::Context) {
        for object in &self.objects {
            // Load vertex arrays
            unsafe {
                gl.enable_vertex_attrib_array(0);
                gl.bind_vertex_array(Some(object.vao));

                // Draw index arrays
                for (ibo, num) in &object.ibos {
                    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(*ibo));
                    gl.draw_elements(glow::TRIANGLES, *num, glow::INT, 0);
                }
            }
        }
    }

    pub fn clean(self, gl: &glow::Context) {
        for object in self.objects {
            object.clean(gl);
        }
    }
}