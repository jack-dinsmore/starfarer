use winit::{event::{
    Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState},
    event_loop::{EventLoop, ControlFlow}};
use crate::{Graphics, Lepton, fps_limiter::FPSLimiter};

pub struct Control {
    pub(crate) event_loop: EventLoop<()>
}

impl Control {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        Control { event_loop }
    }

    /// Run the main game loop. Consumes self.
    pub fn run<L: Lepton>(self, mut graphics: Graphics, mut lepton: L) -> ! {
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

pub struct KeyTracker {
    low_mask: u128,
    high_mask: u128,
}

impl KeyTracker {
    pub fn new() -> KeyTracker {
        KeyTracker {
            low_mask: 0,
            high_mask: 0,
        }
    }

    pub fn key_down(&mut self, vk: winit::event::VirtualKeyCode) {
        if (vk as u32) < 128 {
            self.low_mask |= 1 << (vk as u32);
        } else {
            self.high_mask |= 1 << ((vk as u32) - 128);
        }
    }

    pub fn key_up(&mut self, vk: winit::event::VirtualKeyCode) {
        if (vk as u32) < 128 {
            self.low_mask &= !(1 << (vk as u32));
        } else {
            self.high_mask &= !(1 << ((vk as u32) - 128));
        }
    }

    pub fn get_state(&self, vk: winit::event::VirtualKeyCode) -> bool{
        0 != if (vk as u32) < 128 {
            self.low_mask & (1 << (vk as u32))
        } else {
            self.high_mask & (1 << ((vk as u32) - 128))
        }
    }
}