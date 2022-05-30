use cgmath::{Matrix4, Point3, Vector3, Deg};
use vk_shader_macros::include_glsl;

use crate::shader::ShaderData;
use crate::Graphics;

/// An example shader, made for use with a camera.
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct CameraData {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}
impl CameraData {

    pub fn update(&mut self, delta_time: f32) {
        
    }
}

impl ShaderData for CameraData {
    const VERTEX_CODE: &'static [u32] = include_glsl!("src/shader/26-shader-depth.vert", kind: vert);
    const FRAGMENT_CODE: &'static [u32] = include_glsl!("src/shader/26-shader-depth.frag", kind: frag);

    fn default() -> Self {
        CameraData {
            model: Matrix4::from_angle_z(Deg(90.0)),
            view: Matrix4::look_at_rh(
                Point3::new(2.0, 2.0, 2.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ),
            proj: {
                let mut proj = cgmath::perspective(Deg(45.0), 1.0, 0.1, 10.0);
                proj[1][1] = proj[1][1] * -1.0;
                proj
            },
        }
    }
}

pub struct Camera {
    aspect: f32,
    pos: Point3<f32>,
}

impl Camera {
    pub fn new(graphics: &Graphics) -> Camera {
        Camera {
            aspect: graphics.swapchain_extent.width as f32 / graphics.swapchain_extent.height as f32,
            pos: Point3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn update(&self, camera_data: &mut CameraData) {
        camera_data.model = Matrix4::from_angle_z(Deg(90.0));
        camera_data.view = Matrix4::look_at_rh(
                self.pos + Vector3::new(2.0, 2.0, 2.0),
                self.pos,
                Vector3::new(0.0, 0.0, 1.0),
            );
        camera_data.proj = {
            let mut proj = cgmath::perspective(Deg(45.0), self.aspect, 0.1, 10.0);
            proj[1][1] = proj[1][1] * -1.0;
            proj
        }
    }

    pub fn adjust(&mut self, v: Vector3<f32>) {
        self.pos += v;
    }
}