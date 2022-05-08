use glow::*;
use std::ops::Drop;

pub struct ShaderManager<'a> {
    gl: &'a glow::Context,
    example_shader: NativeProgram,
}

/// Enumerates all the shaders we have coded
pub enum Shader {
    Example(f32),
}

impl Shader {
    fn get_source(self) -> Vec<(u32, &'static str)> {
        match self {
            Example => vec![
                (glow::VERTEX_SHADER, &include_str!("vertex.hlsl")), 
                (glow::FRAGMENT_SHADER, &include_str!("fragment.hlsl"))
            ]
        }
    }

    fn set_uniforms(self, gl: &glow::Context, program: NativeProgram) {
        unsafe{
            match self {
                Self::Example(blue) => {
                    gl.uniform_1_f32(gl.get_uniform_location(program, "blue").as_ref(), blue);
                },
            }
        }
    }
}


impl<'a> ShaderManager<'a> {
    pub fn new(gl: &'a glow::Context) -> ShaderManager<'a> {

        // Initialize the example shader
        let example_shader = Self::init_shader(gl, vec![
            (glow::VERTEX_SHADER, &include_str!("vertex.hlsl")), 
            (glow::FRAGMENT_SHADER, &include_str!("fragment.hlsl"))
        ]);

        ShaderManager { gl, example_shader }
    }

    pub fn load_example(&self) {
        unsafe {
            self.gl.use_program(Some(self.example_shader));
        }
    }

    pub fn set_uniforms(&self, shader: Shader) {
        shader.set_uniforms(self.gl, self.example_shader);
    }

    /// Create a program id for a shader
    fn init_shader(gl: &'a glow::Context, shader_src: Vec<(u32, &'static str)>) -> NativeProgram {
        let mut shaders = Vec::with_capacity(shader_src.len());
        
        unsafe {
            // Make program
            let program = gl.create_program().expect("Cannot create program");

            // Load in shaders
            for (shader_type, shader_source) in shader_src.iter() {
                let shader = gl
                    .create_shader(*shader_type)
                    .expect("Cannot create shader");
                    gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("{}", gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            // Link shader to program
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            // Clean up
            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            program
        }
    }
}

impl<'a> Drop for ShaderManager<'a> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.example_shader);
        }
    }
}