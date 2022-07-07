mod fps_limiter;
mod receiver;
mod renderer;

use winit::{event::{
    Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState},
    event_loop::{EventLoop, ControlFlow}};

pub use fps_limiter::*;
pub use receiver::*;
pub use renderer::*;
use crate::Graphics;


pub struct Backend {
    pub(crate) event_loop: EventLoop<()>
}

impl Backend {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        Backend { event_loop }
    }

    /// Run the main game loop. Consumes self.
    pub fn run<L: Renderer + InputReceiver>(self, mut graphics: Graphics, mut lepton: L) -> ! {
        let mut tick_counter = FPSLimiter::new();

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                | Event::DeviceEvent { event, .. } => {
                    match event {
                        DeviceEvent::MouseMotion{delta} => if lepton.mouse_motion(delta) {
                            graphics.center_cursor();
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
                    let delta_time = tick_counter.delta_time();

                    lepton.update(delta_time);
                    match graphics.begin_frame() {
                        Some(mut data) => {
                            lepton.render(&graphics, &mut data);
                            graphics.end_frame(data);
                        },
                        None => ()
                    };
                    tick_counter.tick_frame();
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