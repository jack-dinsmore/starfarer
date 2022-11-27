mod camera;
mod lights;

use cgmath::{Matrix4};
use vk_shader_macros::include_glsl;

use crate::shader::{Signature, vertex};
use crate::input::{InputType, InputLevel};
pub use camera::*;
pub use lights::*;

#[repr(C)]
pub struct ObjectPushConstants {
    pub model: Matrix4<f32>,
    pub rotation: Matrix4<f32>,
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

pub struct EmptyPushConstants {}


pub struct ModelSignature;
impl Signature for ModelSignature {
    type V = vertex::VertexModel;
    type PushConstants = ObjectPushConstants;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/builtin/tex.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/builtin/tex.frag", kind: frag);
    const INPUTS: &'static [InputType] = &[
        InputType::Camera,
        InputType::Lights,
        InputType::Texture{level: InputLevel::Model},
    ];
}

pub struct UISignature;
impl Signature for UISignature {
    type V = vertex::Vertex2Tex;
    type PushConstants = UIPushConstants;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/builtin/ui.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/builtin/ui.frag", kind: frag);
    const INPUTS: &'static [InputType] = &[
        InputType::Texture{level: InputLevel::Model},
    ];
}

pub struct LPSignature;
impl Signature for LPSignature {
    type V = vertex::VertexLP;
    type PushConstants = ObjectPushConstants;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/builtin/lp.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/builtin/lp.frag", kind: frag);const INPUTS: &'static [InputType] = &[
        InputType::Camera,
        InputType::Lights,
        InputType::Texture{level: InputLevel::Model},
    ];
}