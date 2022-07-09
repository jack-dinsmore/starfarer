use cgmath::{Matrix4, Point3, Vector4, Vector3, Deg};

use crate::Graphics;
use crate::constants::PI;
use crate::shader;

const NUM_LIGHTS: usize = 2; // Same as NUM_LIGHTS in shaders

/// An example shader, made for use with a camera.
pub struct Camera {
    aspect: f32,
    pos: Point3<f32>,
    theta: f32,
    phi: f32,
    input: shader::Input,
}


impl Camera {
    pub fn new(graphics: &Graphics, pos: Point3<f32>) -> Camera {
        let input = shader::InputType::Camera.new(graphics);

        Camera {
            aspect: graphics.swapchain_extent.width as f32 / graphics.swapchain_extent.height as f32,
            pos,
            theta: PI as f32 / 2.0,
            phi: 0.0,
            input,
        }
    }

    pub fn update_input(&mut self, buffer_index: usize) {
        let data = shader::builtin::CameraData {
            view: Matrix4::look_at_rh(
                self.pos,
                self.pos - Vector3::new(
                    f32::sin(self.theta) * f32::cos(self.phi),
                    f32::sin(self.theta) * f32::sin(self.phi),
                    f32::cos(self.theta)),
                Vector3::new(0.0, 0.0, 1.0),
            ),
            proj: {
                let mut proj = cgmath::perspective(Deg(45.0), self.aspect, 0.1, 10.0);
                proj[1][1] = proj[1][1] * -1.0;
                proj
            },
            camera_pos: Vector4::new(self.pos.x, self.pos.y, self.pos.z, 0.0)
        };
        self.input.update(data, buffer_index);
    }

    pub fn adjust(&mut self, v: Vector3<f32>) {
        self.pos += v;
    }

    pub fn turn(&mut self, delta_theta: f32, delta_phi: f32) {
        self.theta = f32::min(f32::max(self.theta + delta_theta, 1e-5), PI as f32- 1e-5);
        self.phi = (self.phi + delta_phi) % (2.0 * PI as f32);
    }
}