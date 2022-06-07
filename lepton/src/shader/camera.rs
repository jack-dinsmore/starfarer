use cgmath::{Matrix4, Point3, Vector4, Vector3, Deg};
use vk_shader_macros::include_glsl;

use crate::shader::ShaderData;
use crate::Graphics;
use crate::constants::PI;

const NUM_LIGHTS: usize = 2; // Same as NUM_LIGHTS in shaders

/// An example shader, made for use with a camera.
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct CameraData {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
    pub camera_pos: Vector4<f32>,
    pub light_pos: [Vector4<f32>; NUM_LIGHTS],
    pub light_features: [Vector4<f32>; NUM_LIGHTS],
    pub num_lights: u32,
}

impl ShaderData for CameraData {
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/tex.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/tex.frag", kind: frag);

    fn default() -> Self {
        let camera_pos = Point3::new(0.0, 0.0, 0.0);
        let mut light_pos = [Vector4::new(0.0, 0.0, 0.0, 0.0); NUM_LIGHTS];
        let mut light_features = [Vector4::new(0.0, 0.0, 0.0, 0.0); NUM_LIGHTS];

        light_pos[0] = Vector4::new(5.0, -5.0, 10.0, 0.0);
        light_features[0] = Vector4::new(0.5, 0.5, 2.0, 0.0);

        CameraData {
            model: Matrix4::from_angle_z(Deg(90.0)),
            view: Matrix4::look_at_rh(
                camera_pos,
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ),
            proj: {
                let mut proj = cgmath::perspective(Deg(45.0), 1.0, 0.1, 10.0);
                proj[1][1] = proj[1][1] * -1.0;
                proj
            },
            camera_pos: Vector4::new(camera_pos.x, camera_pos.y, camera_pos.z, 0.0),
            light_pos,
            light_features,
            num_lights: 1,
        }
    }
}

pub struct Camera {
    aspect: f32,
    pos: Point3<f32>,
    theta: f32,
    phi: f32,
}

impl Camera {
    pub fn new(graphics: &Graphics) -> Camera {
        Camera {
            aspect: graphics.swapchain_extent.width as f32 / graphics.swapchain_extent.height as f32,
            pos: Point3::new(2.0, 0.0, 2.0),
            theta: PI as f32 / 2.0,
            phi: 0.0,
        }
    }

    pub fn update(&self, camera_data: &mut CameraData) {
        camera_data.model = Matrix4::from_angle_z(Deg(90.0));
        camera_data.view = Matrix4::look_at_rh(
                self.pos,
                self.pos - Vector3::new(
                    f32::sin(self.theta) * f32::cos(self.phi),
                    f32::sin(self.theta) * f32::sin(self.phi),
                    f32::cos(self.theta)),
                Vector3::new(0.0, 0.0, 1.0),
            );
        camera_data.proj = {
            let mut proj = cgmath::perspective(Deg(45.0), self.aspect, 0.1, 10.0);
            proj[1][1] = proj[1][1] * -1.0;
            proj
        };
        camera_data.camera_pos = Vector4::new(self.pos.x, self.pos.y, self.pos.z, 0.0);
    }

    pub fn adjust(&mut self, v: Vector3<f32>) {
        self.pos += v;
    }

    pub fn turn(&mut self, delta_theta: f32, delta_phi: f32) {
        self.theta = f32::min(f32::max(self.theta + delta_theta, 1e-5), PI as f32- 1e-5);
        self.phi = (self.phi + delta_phi) % (2.0 * PI as f32);
    }
}