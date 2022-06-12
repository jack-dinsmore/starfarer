use vk_shader_macros::include_glsl;
use std::rc::Rc;

use crate::{Graphics, Pattern, RenderData};
use crate::model::{Model, VertexType, TextureType, primitives::Vertex2Tex};
use crate::shader::{Signature, InputType};

pub type Color = [f64; 3];

pub mod color {
    use super::Color;

    const RED: Color = [1.0, 0.0, 0.0];
}

struct UISignature;
impl Signature for UISignature {
    type V = Vertex2Tex;
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/builtin/ui.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/builtin/ui.frag", kind: frag);
    const INPUTS: &'static [InputType] = &[];
}

pub struct UserInterface {
    //pattern: Pattern<UISignature>,
    //model: Rc<Model>,
}

impl UserInterface {
    pub fn new(graphics: &mut Graphics) -> Self {
        /*let mut pattern = Pattern::<UISignature>::begin(graphics);
        let vertices = vec![
            Vertex2Tex{ pos: [1.0, 0.0, 0.0, 0.0], tex_coord: [0.0, 0.0]}, // Upper left
            Vertex2Tex{ pos: [-1.0, 0.0, 0.0, 0.0], tex_coord: [1.0, 0.0]}, // Upper right
            Vertex2Tex{ pos: [1.0, -0.0, 0.0, 0.0], tex_coord: [0.0, 1.0]}, // Lower left
            Vertex2Tex{ pos: [-1.0, -0.0, 0.0, 0.0], tex_coord: [1.0, 1.0]}, // Lower right
        ];
        let indices = vec![0, 1, 2, 1, 3, 2];
        let model = Model::new(graphics, &pattern, VertexType::Specified2Tex(vertices, indices), TextureType::Blank)
            .expect("Could not create UI");
        pattern.add_model(model.clone());
        let pattern = pattern.end(graphics);*/

        Self {
            //pattern,
            //model,
        }
    }

    pub fn render(&self, render_data: &mut RenderData) {
        //self.pattern.render(render_data);
    }

    pub fn check_reload(&mut self, graphics: &Graphics) {
        //self.pattern.check_reload(graphics);
    }
}