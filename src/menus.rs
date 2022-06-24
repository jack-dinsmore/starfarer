use std::rc::Rc;

use lepton::prelude::*;

pub struct Common {
    font: Rc<Font>,
    blank: Rc<Model>,
}

impl Common {
    pub fn new(graphics: &Graphics, shader: &Shader<builtin::UISignature>) -> Self {
        let vertices = vec![
            vertex::Vertex2Tex { pos: [-1.0, -1.0], tex_coord: [0.0, 0.0]},
            vertex::Vertex2Tex { pos: [1.0, -1.0], tex_coord: [1.0, 0.0]},
            vertex::Vertex2Tex { pos: [-1.0, 1.0], tex_coord: [0.0, 1.0]},
            vertex::Vertex2Tex { pos: [1.0, 1.0], tex_coord: [1.0, 1.0]},
        ];
        let indices = vec![ 0, 2, 1, 1, 2, 3 ];
        Self {
            font: Rc::new(Font::new(graphics, shader, "Roboto-Regular", 48)),
            blank: Rc::new(Model::new(graphics, shader, VertexType::Specified(vertices, indices), TextureType::Blank).expect("Could not load blank model")),
        }
    }
}

pub struct FPS {
    time: f32,
    short_time: i32,
    frames: u32,
    fps: u32,
    elements: Vec<Element>,
}

impl UserInterface for FPS {
    fn get_elements(&self) -> &Vec<Element> { return &self.elements }
}

impl FPS {
    pub fn new(common: &Common) -> Self {
        let elements = vec![
            Element::new_text(common.font.clone(), "FPS:".to_owned(), -1.0, -1.0),
        ];

        Self {
            time: 0.0,
            short_time: 0,
            frames: 0,
            fps: 0,
            elements,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        self.frames += 1;
        if self.time as i32 != self.short_time {
            // Update FPS
            self.fps = self.frames;
            self.frames = 0;
            self.short_time = self.time as i32;
        }
        match &mut self.elements[0] {
            Element::Text(_, t, _, _) => { *t = format!("FPS: {}", self.fps); },
            _ => panic!(""),
        };
    }
}


pub struct Escape {
    elements: Vec<Element>,
    pub is_open: bool
}

impl UserInterface for Escape {
    fn get_elements(&self) -> &Vec<Element> { return &self.elements }
}

impl Escape {
    pub fn new(common: &Common) -> Self {
        let elements = vec![
            Element::new_background(common.blank.clone(), 0.0, 0.0, 0.17, 0.10),
            Element::new_button(common.font.clone(), common.blank.clone(), "Settings".to_owned(), 0.0, 0.0, 0.15, 0.08),
        ];

        Self {
            elements,
            is_open: false,
        }
    }
}