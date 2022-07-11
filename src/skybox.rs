use lepton::prelude::*;
use std::rc::Rc;
use vk_shader_macros::include_glsl;

pub struct SkyboxSignature {}
impl shader::Signature for SkyboxSignature {
    type V = vertex::Vertex3Tex;
    type PushConstants = builtin::EmptyPushConstants;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shaders/skybox.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shaders/skybox.frag", kind: frag);
    const INPUTS: &'static [shader::InputType] = &[
        shader::InputType::Camera,
    ];
}

pub struct Skybox {
    pub skybox_shader: Shader<SkyboxSignature>,
    pub model: Rc<Model>,
}

impl Skybox {
    pub fn from_temp(graphics: &mut Graphics) -> Self {
        let skybox_shader = Shader::new(graphics);
        let model = Rc::new(Model::new(graphics, &skybox_shader,
            VertexType::<vertex::VertexModel>::skybox(),
            TextureType::Transparency(include_bytes!("../assets/temp/skybox.png")))
            .expect("Model creation failed"));

        Self {
            skybox_shader,
            model,
        }
    }
}