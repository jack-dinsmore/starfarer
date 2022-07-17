#![allow(clippy::match_single_binding)]
#![allow(clippy::single_match)]

mod fps_limiter;
mod receiver;
mod renderer;

use winit::{event::{
    Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState},
    event_loop::{EventLoop, ControlFlow}};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

pub use fps_limiter::*;
pub use receiver::*;
pub use renderer::*;
use crate::{Graphics, GraphicsData};
use crate::physics::{Physics, PhysicsData};


pub struct Backend {
    pub(crate) event_loop: EventLoop<()>,
    pub(crate) graphics_data_sender: Option<Sender<GraphicsData>>,
    pub(crate) graphics_data_receiver: Option<Receiver<GraphicsData>>,
    pub(crate) physics_data_sender: Option<Sender<PhysicsData>>,
    pub(crate) physics_data_receiver: Option<Receiver<PhysicsData>>,
}

impl Backend {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let graphics_data = mpsc::channel();
        let physics_data = mpsc::channel();
        Backend {
            event_loop,
            graphics_data_sender: Some(graphics_data.0),
            graphics_data_receiver: Some(graphics_data.1),
            physics_data_sender: Some(physics_data.0),
            physics_data_receiver: Some(physics_data.1),
        }
    }

    /// Run the main game loop. Consumes self.
    pub fn run<L: Renderer + InputReceiver>(mut self, mut graphics: Graphics, mut lepton: L) -> ! {
        let mut physics = Physics::new(&mut self);

        // Add objects to graphics and physics
        graphics.object_models = lepton.load_models(&graphics);
        physics.rigid_bodies = lepton.load_rigid_bodies();
        
        // Validate the receivers and senders
        if self.graphics_data_sender.is_some() {
            panic!("The physics engine did not pick up the render data sender");
        }
        if self.graphics_data_receiver.is_some() {
            panic!("The graphics engine did not pick up the render data receiver");
        }
        let physics_data_sender = match self.physics_data_sender.take() {
            Some(t) => t,
            None => panic!("Someone picked up the physics data sender"),
        };
        if self.physics_data_receiver.is_some() {
            panic!("The physics engine did not pick up the physics data receiver");
        }

        // Begin initialization of threads
        let mut graphics_limiter = FPSLimiter::with_limits(None, None);
        let mut physics_limiter = FPSLimiter::with_limits(Some(60), Some(2));
        let mut first_mouse_motion = true;
        lepton.prepare(&graphics);
        
        // Spawn the physics engine
        thread::spawn(move || {
            loop {
                let delta_time = physics_limiter.tick_frame();
                physics.update(delta_time);
            }
        });

        // Spawn the physics engine
        self.event_loop.run(move |event, _, control_flow| {
            match event {
                | Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion{mut delta} => {
                            #[cfg(target_os = "macos")]
                            {
                                if !first_mouse_motion {
                                    delta = (delta.0 + graphics.last_delta.0, delta.1 + graphics.last_delta.1);
                                    graphics.last_delta = (0.0, 0.0);
                                }
                            }
                            first_mouse_motion = false;

                            if lepton.mouse_motion(delta) {
                                graphics.center_cursor();
                            }
                        },
                        _ => ()
                    }
                }
                | Event::WindowEvent { event, ..} => {
                    match event {
                        | WindowEvent::CursorMoved {position, ..} => {
                            graphics.mouse_position = (
                                position.x as f32 / graphics.window_width as f32 * 2.0 - 1.0,
                                position.y as f32 / graphics.window_height as f32 * 2.0 - 1.0
                            );
                        }
                        | WindowEvent::CloseRequested => {
                            graphics.terminate();
                            *control_flow = ControlFlow::Exit
                        },
                        | WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                KeyboardInput { virtual_keycode, state, .. } => {
                                    match state {
                                        ElementState::Pressed =>
                                            if let Some(vk) = virtual_keycode{
                                                lepton.key_down(vk);
                                            },
                                        ElementState::Released =>
                                            if let Some(vk) = virtual_keycode{
                                                lepton.key_up(vk);
                                            },
                                    }
                                    
                                },
                            }
                        },
                        | WindowEvent::MouseInput { state, button, .. } => {
                            match state {
                                ElementState::Pressed => {
                                    lepton.mouse_down(graphics.mouse_position, button);
                                }
                                ElementState::Released => ()
                            }
                        },
                        | WindowEvent::Resized(new_size) => {
                            graphics.terminate();
                            graphics.resize_framebuffer(new_size.width, new_size.height);
                            lepton.resize(&graphics);
                        },
                        | _ => {},
                    }
                },
                | Event::MainEventsCleared => {
                    graphics.request_redraw();
                },
                | Event::RedrawRequested(_window_id) => {
                    let delta_time = graphics_limiter.tick_frame();
                    graphics.receive();
                    physics_data_sender.send(lepton.update(&graphics, delta_time)).unwrap();
                    match graphics.begin_frame() {
                        Some(data) => {
                            let tasks = lepton.render(&graphics, data.buffer_index);
                            graphics.record(data.buffer_index, tasks);
                            graphics.render(data);
                        },
                        None => ()
                    };
                },
                | Event::LoopDestroyed => {
                    graphics.terminate();
                },
                _ => (),
            };

            if lepton.should_quit() {
                graphics.terminate();
                *control_flow = ControlFlow::Exit;
            }
        });
    }
}