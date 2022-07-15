use cgmath::{Matrix4, Matrix3, Point3, Vector4, Vector3, Deg, EuclideanSpace, Rad};

use crate::Graphics;
use crate::constants::PI;
use crate::shader;

const NUM_LIGHTS: usize = 2; // Same as NUM_LIGHTS in shaders
const MIN_DISTANCE: f32 = 0.1;
const MAX_DISTANCE: f32 = 10_000.0;
const UP: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);

/// An example shader, made for use with a camera.
pub struct Camera {
    aspect: f32,
    pos: Vector3<f32>,
    local_rot: Option<Matrix3<f32>>,
    theta: f32,
    phi: f32,
    input: shader::Input,
}


impl Camera {
    pub fn new(graphics: &Graphics, pos: Vector3<f32>) -> Camera {
        let input = shader::InputType::Camera.new(graphics);

        Camera {
            aspect: graphics.swapchain_extent.width as f32 / graphics.swapchain_extent.height as f32,
            pos,
            local_rot: None,
            theta: PI as f32 / 2.0,
            phi: 0.0,
            input,
        }
    }

    pub fn update_input(&mut self, buffer_index: usize) {
        let view = match self.local_rot {
            Some(rotation) => Matrix4::look_at_rh(
                Point3::from_vec(self.pos),
                Point3::from_vec(self.pos - rotation * Vector3::new(
                    f32::sin(self.theta) * f32::cos(self.phi),
                    f32::sin(self.theta) * f32::sin(self.phi),
                    f32::cos(self.theta))),
                rotation * UP,
            ),
            None => Matrix4::look_at_rh(
                Point3::from_vec(self.pos),
                Point3::from_vec(self.pos - Vector3::new(
                    f32::sin(self.theta) * f32::cos(self.phi),
                    f32::sin(self.theta) * f32::sin(self.phi),
                    f32::cos(self.theta))),
                UP,
            ),
        };
        let data = shader::builtin::CameraData {
            view,
            proj: {
                let mut proj = cgmath::perspective(Deg(45.0), self.aspect, MIN_DISTANCE, MAX_DISTANCE);
                proj[1][1] = proj[1][1] * -1.0;
                proj
            },
            camera_pos: Vector4::new(self.pos.x, self.pos.y, self.pos.z, 0.0),
        };
        self.input.update(data, buffer_index);
    }

    pub fn adjust(&mut self, v: Vector3<f32>) {
        self.pos += v;
    }

    pub fn set_pos(&mut self, pos: Vector3<f32>) {
        self.pos = pos;
    }

    pub fn set_local_rot(&mut self, rot: Matrix3<f32>) {
        self.local_rot = Some(rot);
    }

    pub fn get_rotation(&self) -> Matrix3<f32> {
        Matrix3::from_angle_z(Rad(self.phi)) * Matrix3::from_angle_y(Rad(self.theta - PI as f32 / 2.0))
    }

    pub fn turn(&mut self, delta_theta: f32, delta_phi: f32) {
        self.theta = f32::min(f32::max(self.theta + delta_theta, 1e-5), PI as f32- 1e-5);
        self.phi = (self.phi + delta_phi) % (2.0 * PI as f32);
    }
}