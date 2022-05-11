use glow::*;
use wavefront_obj::{obj};
use log::info;
use num::ToPrimitive;

type IndexType = u8;
const INDEX_TYPE: u32 = glow::UNSIGNED_BYTE;

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
            triangle_vertices.push(v.z);
            triangle_vertices.push(v.y); 
        }

        println!("{:?}", triangle_vertices);

        // We construct a buffer and upload the data
        let vbo = make_buffer(gl, glow::ARRAY_BUFFER, &triangle_vertices, glow::STATIC_DRAW);

        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let mut ibos = Vec::new();
        for geo in object.geometry {
            let mut indices = Vec::new();

            for shape in geo.shapes {
                if let obj::Primitive::Triangle(i1, i2, i3) = shape.primitive {
                    indices.push(i1.0 as IndexType);
                    indices.push(i2.0 as IndexType);
                    indices.push(i3.0 as IndexType);
                    //tex_indices.push(i1.1 as u32);
                    //tex_indices.push(i2.1 as u32);
                    //tex_indices.push(i3.1 as u32);
                    //normal_indices.push(i1.2 as u32);
                    //normal_indices.push(i2.2 as u32);
                    //normal_indices.push(i3.2 as u32);
                }
                else {
                    panic!("Cannot load non-triangle objects")
                }

            }
            println!("{:?}", indices);
            ibos.push((make_buffer(gl, glow::ELEMENT_ARRAY_BUFFER, &indices, glow::STATIC_DRAW), indices.len() as i32));
        }

        Object { vao, vbo, ibos }
    }

    fn clean(self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);

            for (ibo, _) in self.ibos {
                println!("Deleted");
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
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(object.vbo));

                
                /*println!("{:?}", read_buffer(gl, object.vbo, glow::ARRAY_BUFFER, 4));
                println!("{:?}", to_f32(read_buffer(gl, object.vbo, glow::ARRAY_BUFFER, 4)));*/

                gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

                // Draw index arrays
                for (ibo, num) in &object.ibos {
                    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(*ibo));
                    println!("{:?}", read_buffer(gl, *ibo, glow::ELEMENT_ARRAY_BUFFER, 3));

                    gl.draw_elements(glow::TRIANGLES, *num, INDEX_TYPE, 0);
                    //gl.draw_arrays(glow::TRIANGLES, 0, 3);
                }

                gl.disable_vertex_attrib_array(0);
            }
        }
    }

    pub fn clean(self, gl: &glow::Context) {
        for object in self.objects {
            object.clean(gl);
        }
    }
}

unsafe fn make_buffer<T>(gl: &glow::Context, buffer_type: u32, vec: &Vec<T>, draw_type: u32) -> NativeBuffer {
    let vec_u8: &[u8] = core::slice::from_raw_parts(
        vec.as_ptr() as *const u8,
        vec.len() * core::mem::size_of::<T>(),
    );
    let bo = gl.create_buffer().unwrap();
    gl.bind_buffer(buffer_type, Some(bo));
    gl.buffer_data_u8_slice(buffer_type, vec_u8, draw_type);
    bo
}

/// Read a buffer into a vector for debugging purposes
fn read_buffer(gl: &glow::Context, target: NativeBuffer, buffer_type: u32, length: usize) -> Vec<u8> {
    // Inefficiently create a vector with the right size by defining all entries.
    let mut dest_u8 = vec![0; length];
    unsafe {
        gl.bind_buffer(buffer_type, Some(target));
        gl.get_buffer_sub_data(buffer_type, 0, &mut dest_u8);
    }
    dest_u8
}

fn to_f32(vec_u8: Vec<u8>) -> Vec<f32> {
    let mut bytes = [0; 4];
    let mut out = Vec::new();
    assert_eq!(vec_u8.len() % 4, 0);
    for this_f in vec_u8.chunks(4).collect::<Vec<_>>() {
        bytes[0] = this_f[0];
        bytes[1] = this_f[1];
        bytes[2] = this_f[2];
        bytes[3] = this_f[3];
        out.push(f32::from_ne_bytes(bytes));
    }
    out
}