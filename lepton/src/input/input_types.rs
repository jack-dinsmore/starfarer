use ash::vk;
use std::mem::size_of;

use crate::shader::{Data, vertex::Vertex, vertex::Vertex3Tex, ShaderStages, builtin, Signature};

const BLANK_WIDTH: u32 = 4;
const BLANK_HEIGHT: u32 = 4;
const ALMOST_ONE: f32 = 0.999;

/// The level at which the input is loaded
#[derive(Clone, Copy)]
pub enum InputLevel {
    /// The input is only loaded when the shader is loaded
    Shader,

    /// The input is reloaded every model
    Model,
}

/// Names for specific UBOs to be cited by Signatures
pub enum InputType {
    UI,
    Texture{level: InputLevel},
    Camera,
    Lights,
    Custom {
        size: usize,
        shader_stages: ShaderStages,
        level: InputLevel,
    },
}

pub enum VertexType<'a, V: Vertex> {
    Specified(Vec<V>, Vec<u32>),
    Compiled(&'a [u8], usize),
}

impl<'a, V: Vertex> VertexType<'a, V> {
    pub(crate) fn to_vectors(bytes: &'a [u8], num_indices: usize) -> (Vec<V>, Vec<u32>) {
        let vertex_bytes = bytes.len() - num_indices * std::mem::size_of::<u32>();
        unsafe {(
            std::slice::from_raw_parts((&bytes[0] as *const u8) as *const V,
                (vertex_bytes / std::mem::size_of::<V>()) as usize).to_vec(),
            std::slice::from_raw_parts((&bytes[vertex_bytes] as *const u8) as *const u32,
                num_indices).to_vec()
        )}
    }

    pub fn skybox<'b>() -> VertexType::<'b, Vertex3Tex> {
        let vertices = vec![
            Vertex3Tex { pos: [-ALMOST_ONE, ALMOST_ONE, ALMOST_ONE], coord: [0.0, 0.0]},
            Vertex3Tex { pos: [-ALMOST_ONE, -ALMOST_ONE, ALMOST_ONE], coord: [0.499, 0.0]},
            Vertex3Tex { pos: [ALMOST_ONE, ALMOST_ONE, ALMOST_ONE], coord: [0.0, 0.333]},
            Vertex3Tex { pos: [ALMOST_ONE, -ALMOST_ONE, ALMOST_ONE], coord: [0.499, 0.333]},
            Vertex3Tex { pos: [ALMOST_ONE, ALMOST_ONE, -ALMOST_ONE], coord: [0.0, 0.6666]},
            Vertex3Tex { pos: [ALMOST_ONE, -ALMOST_ONE, -ALMOST_ONE], coord: [0.499, 0.6673]},
            Vertex3Tex { pos: [-ALMOST_ONE, ALMOST_ONE, -ALMOST_ONE], coord: [0.0, 1.0]},
            Vertex3Tex { pos: [-ALMOST_ONE, -ALMOST_ONE, -ALMOST_ONE], coord: [0.499, 1.0]},
            Vertex3Tex { pos: [ALMOST_ONE, -ALMOST_ONE, ALMOST_ONE], coord: [1.0, 0.6673]},
            Vertex3Tex { pos: [-ALMOST_ONE, -ALMOST_ONE, ALMOST_ONE], coord: [1.0, 1.0]},

            Vertex3Tex { pos: [-ALMOST_ONE, -ALMOST_ONE, -ALMOST_ONE], coord: [0.501, 0.0]},
            Vertex3Tex { pos: [-ALMOST_ONE, -ALMOST_ONE, ALMOST_ONE], coord: [1.0, 0.0]},
            Vertex3Tex { pos: [-ALMOST_ONE, ALMOST_ONE, -ALMOST_ONE], coord: [0.501, 0.333]},
            Vertex3Tex { pos: [-ALMOST_ONE, ALMOST_ONE, ALMOST_ONE], coord: [1.0, 0.333]},
            Vertex3Tex { pos: [ALMOST_ONE, ALMOST_ONE, -ALMOST_ONE], coord: [0.501, 0.666]},
            Vertex3Tex { pos: [ALMOST_ONE, ALMOST_ONE, ALMOST_ONE], coord: [1.0, 0.666]},
        ];
        let indices = vec![
            0, 2, 1, 1, 2, 3,
            2, 4, 3, 3, 4, 5,
            4, 6, 5, 5, 6, 7,
            5, 7, 8, 8, 7, 9,
            10, 12, 11, 11, 12, 13,
            12, 14, 13, 13, 14, 15,
        ];
        VertexType::<'b, Vertex3Tex>::Specified(vertices, indices)
    }
}

pub enum TextureType<'a> {
    Mipmap(&'a [u8]),
    Transparency(&'a [u8]),
    Monochrome(&'a [u8]),
    Blank,
}

impl<'a> TextureType<'a> {
    pub(crate) fn to_image(bytes: &'a [u8]) -> image::DynamicImage {
        image::load_from_memory(bytes).unwrap()
    }
}

impl InputType {
    /// Make a new UBO and store it in graphics. Must be permanent.
    pub const fn make_custom<D: Data, S: Signature>() -> InputType {
        InputType::Custom {
            size: size_of::<D>(),
            shader_stages: D::STAGES,
            level: D::LEVEL,
        }
    }

    pub(crate) fn get_size(&self) -> u32 {
        match self {
            InputType::UI => 0,
            InputType::Texture{..} => 0,
            InputType::Camera => size_of::<builtin::CameraData>() as u32,
            InputType::Lights => size_of::<builtin::LightsData>() as u32,
            InputType::Custom{size, ..} => *size as u32,
        }
    }

    pub(crate) fn get_stages(&self) -> vk::ShaderStageFlags {
        vk::ShaderStageFlags::from_raw(match self {
            InputType::UI => 0,
            InputType::Texture{..} => ShaderStages::FRAGMENT.f,
            InputType::Camera => builtin::CameraData::STAGES.f,
            InputType::Lights => builtin::LightsData::STAGES.f,
            InputType::Custom{shader_stages, ..} => shader_stages.f,
        })
    }

    pub(crate) fn get_descriptor_type(&self) -> vk::DescriptorType {
        match &self {
            InputType::UI | InputType::Camera | InputType::Lights | InputType::Custom{..} => vk::DescriptorType::UNIFORM_BUFFER,
            InputType::Texture{..} => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        }
    }

    pub(crate) fn get_level(&self) -> InputLevel {
        match &self {
            InputType::UI => InputLevel::Shader,
            InputType::Texture{level} => *level,
            InputType::Camera => builtin::CameraData::LEVEL,
            InputType::Lights => builtin::LightsData::LEVEL,
            InputType::Custom{level, ..} => *level,
        }
    }
}