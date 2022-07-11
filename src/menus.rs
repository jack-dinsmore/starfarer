use std::rc::Rc;

use lepton::prelude::*;
use starfarer_macros::include_font;

pub struct Common {
    font: Rc<Font>,
    blank: Rc<Model>,
}

impl Common {
    pub fn new(graphics: &Graphics, shader: &Shader<builtin::UISignature>) -> Self {
        let vertices = vec![
            vertex::Vertex2Tex { pos: [0.0, 0.0], coord: [0.0, 0.0]},
            vertex::Vertex2Tex { pos: [1.0, 0.0], coord: [1.0, 0.0]},
            vertex::Vertex2Tex { pos: [0.0, 1.0], coord: [0.0, 1.0]},
            vertex::Vertex2Tex { pos: [1.0, 1.0], coord: [1.0, 1.0]},
        ];
        let indices = vec![ 0, 2, 1, 1, 2, 3 ];
        Self {
            font: Rc::new(Font::new(graphics, shader, include_font!("../assets/fonts/NunitoSans/NunitoSans-Bold.ttf", 24), 24, 3)),
            blank: Rc::new(Model::new(graphics, shader, VertexType::Specified(vertices, indices), TextureType::Blank).expect("Could not load blank model")),
        }
    }
}

pub struct FPS {
    time: f32,
    short_time: i32,
    frames: u32,
    fps: u32,
}

impl FPS {
    pub fn new(common: &Common) -> UserInterface<Self> {
        UserInterface::new(
            Self {
                time: 0.0,
                short_time: 0,
                frames: 0,
                fps: 0,
            }
        ).add(Element::Text{ 
            font: common.font.clone(),
            text: "FPS:".to_owned(),
            color: color::WHITE,
            x: -1.0,
            y: -1.0,    
        })
    }

    pub fn update(&mut self, delta_time: f32, elements: &mut Vec<ElementData<FPS>>) {
        self.time += delta_time;
        self.frames += 1;
        if self.time as i32 != self.short_time {
            // Update FPS
            self.fps = self.frames;
            self.frames = 0;
            self.short_time = self.time as i32;
        }
        
        match &mut elements[0] {
            ElementData::Text { text, .. } => { *text = format!("FPS: {}", self.fps); },
            _ => panic!(""),
        };
    }
}


pub struct Escape {
    pub is_open: bool,
    pub quit: bool,
}

impl Escape {
    pub fn new(common: &Common) -> UserInterface<Self> {
        UserInterface::new(
            Self {
                is_open: false,
                quit: false,
            }
        ).add(Element::Background{
            blank: common.blank.clone(),
            x: 0.0,
            y: 0.0,
            width: 0.17,
            height: 0.10
        }).add(Element::Button{
            blank: common.blank.clone(),
            font: common.font.clone(),
            text: "SETTINGS".to_owned(),
            x: 0.0,
            y: 0.0,
            width: 0.15,
            height: 0.08,
            action: Box::new(|escape| {
                escape.quit = true;
            }),
        })
    }
}