use cgmath::{Matrix4, Vector4};
use vk_shader_macros::include_glsl;

use crate::{shader, model};

pub const NUM_LIGHTS: usize = 2;

#[repr(C)]
pub struct ObjectPushConstants {
    pub model: Matrix4<f32>,
}

#[repr(C)]
pub struct UIPushConstants {
    pub x: f32,
    pub y: f32,
    pub stretch_x: f32,
    pub stretch_y: f32,
    pub color: [f32; 4],
    pub depth: f32,
}


pub struct ModelSignature;
impl shader::Signature for ModelSignature {
    type V = model::vertex::VertexModel;
    type PushConstants = ObjectPushConstants;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/builtin/tex.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/builtin/tex.frag", kind: frag);
    const INPUTS: &'static [shader::InputType] = &[
        shader::InputType::Camera,
        shader::InputType::Lights,
    ];
}

pub struct UISignature;
impl shader::Signature for UISignature {
    type V = model::vertex::Vertex2Tex;
    type PushConstants = UIPushConstants;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/builtin/ui.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/builtin/ui.frag", kind: frag);
    const INPUTS: &'static [shader::InputType] = &[];
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