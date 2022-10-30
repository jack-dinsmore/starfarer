use lepton::prelude::*;
use std::rc::Rc;
use vk_shader_macros::include_glsl;
use cgmath::{Vector4, Vector3, Zero};

pub struct SkyboxSignature {}
impl shader::Signature for SkyboxSignature {
    type V = vertex::Vertex3Tex;
    type PushConstants = SkyboxPushConstants;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shaders/skybox.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shaders/skybox.frag", kind: frag);
    const INPUTS: &'static [InputType] = &[
        InputType::Camera,
        InputType::Texture{level: InputLevel::Shader},
        InputType::Texture{level: InputLevel::Model},
    ];
}

#[repr(C)]
pub struct SkyboxPushConstants {
    planet_pos: Vector4<f32>,
    sun_pos: Vector4<f32>,
    planet_info: Vector4<f32>, // Zero if allowed, radius, scale height, min_alpha
}

impl SkyboxPushConstants {
    pub fn default() -> Self {
        SkyboxPushConstants { 
            planet_pos: Vector4::zero(),
            sun_pos: Vector4::zero(),
            planet_info: Vector4::zero(),
        }
    }
}

pub struct Skybox {
    pub skybox_shader: Shader<SkyboxSignature>,
    pub push_constants: SkyboxPushConstants,
    pub model: Rc<Model>,
    sky_colors: Input,
}

impl Skybox {
    pub fn from_temp(graphics: &mut Graphics, camera: &builtin::Camera) -> Self {
        let sky_colors = Input::new_texture(graphics, TextureType::Transparency(include_bytes!("../../assets/calc/sky.png")));
        let skybox_shader = Shader::new(graphics, vec![&camera.input, &sky_colors]);
        let model = Rc::new(Model::new(
            graphics, 
            &skybox_shader,
            VertexType::<vertex::VertexModel>::skybox(), 
            vec![Input::new_texture(graphics, TextureType::Transparency(include_bytes!("../../assets/temp/skybox.png")))]
        ).expect("Model creation failed"));

        Self {
            skybox_shader,
            model,
            sky_colors,
            push_constants: SkyboxPushConstants::default(),
        }
    }

    pub fn reset_push_constants(&mut self, planet_pos: Option<Vector3<f32>>, sun_pos: Option<Vector3<f32>>, atmosphere: Option<super::planet::Atmosphere>, radius: f32) {
        self.push_constants = if let Some(planet_pos) = planet_pos {
            if let Some(sun_pos) = sun_pos {
                if let Some(atmosphere) = atmosphere {
                    SkyboxPushConstants {
                        planet_pos: Vector4::new(planet_pos.x, planet_pos.y, planet_pos.z, 0.0),
                        sun_pos: Vector4::new(sun_pos.x, sun_pos.y, sun_pos.z, 0.0),
                        planet_info: Vector4::new(
                            1.0,
                            radius,
                            atmosphere.scale_height,
                            atmosphere.min_alpha
                        ),
                    }
                } else {
                    SkyboxPushConstants::default()
                }
            } else {
                SkyboxPushConstants::default()
            }
        } else {
            SkyboxPushConstants::default()
        };
    }
}