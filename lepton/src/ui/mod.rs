use ash::vk;

pub type Color = [f64; 3];

pub mod color {
    use super::Color;

    const RED: Color = [1.0, 0.0, 0.0];
}

pub struct Surface {
    image: vk::Image,
}

impl Surface {
    pub fn fill(&mut self, color: Color) {
        //self.image.
    }
}