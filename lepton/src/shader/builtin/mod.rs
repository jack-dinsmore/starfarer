use cgmath::{Matrix4, Vector4};
use vk_shader_macros::include_glsl;

use crate::{shader, model};

pub const NUM_LIGHTS: usize = 2;

pub struct TextureShader;
impl shader::Signature for TextureShader {
    type V = model::primitives::VertexModel;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/builtin/tex.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/builtin/tex.frag", kind: frag);
    const INPUTS: &'static [shader::InputType] = &[
        shader::InputType::Object,
        shader::InputType::Camera,
        shader::InputType::Lights,
    ];
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ObjectData {
    pub model: Matrix4<f32>,
}
impl shader::Data for ObjectData {
    const BINDING: u32 = 0;
    const STAGES: shader::ShaderStages = shader::ShaderStages::VERTEX;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CameraData {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
    pub camera_pos: Vector4<f32>,
}
impl shader::Data for CameraData {
    const BINDING: u32 = 1;
    const STAGES: shader::ShaderStages = shader::ShaderStages::VERTEX.and(shader::ShaderStages::FRAGMENT);
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightsData {
    pub light_pos: [Vector4<f32>; NUM_LIGHTS],
    pub light_features: [Vector4<f32>; NUM_LIGHTS],
    pub num_lights: u32,
}
impl shader::Data for LightsData {
    const BINDING: u32 = 2;
    const STAGES: shader::ShaderStages = shader::ShaderStages::FRAGMENT;
}