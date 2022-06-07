use winit::{event::{
    Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState},
    event_loop::{EventLoop, ControlFlow}};
use crate::{Graphics, PatternTrait, fps_limiter::FPSLimiter};

pub struct Control {
    pub(crate) event_loop: EventLoop<()>
}

impl Control {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        Control { event_loop }
    }

    /// Run the main game loop. Consumes self.
    pub fn run<L: Lepton>(self, mut graphics: Graphics, mut lepton: L, print_fps: bool) -> ! {
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
                                                if lepton.keydown(vk) {
                                                    graphics.terminate();
                                                    *control_flow = ControlFlow::Exit
                                                }
                                            },
                                        ElementState::Released =>
                                            if let Some(vk) = virtual_keycode{
                                                if lepton.keyup(vk) {
                                                    graphics.terminate();
                                                    *control_flow = ControlFlow::Exit
                                                }
                                            },
                                    }
                                    
                                },
                            }
                        },
                        | WindowEvent::Resized(new_size) => {
                            graphics.terminate();
                            graphics.resize_framebuffer(new_size.width, new_size.height);
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
                    graphics.draw_frame(lepton.get_pattern());
                    
                    if print_fps {
                        print!("FPS: {}\r", tick_counter.fps());
                    }
                    tick_counter.tick_frame();
                },
                | Event::LoopDestroyed => {
                    graphics.terminate();
                },
                _ => (),
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

    pub fn keydown(&mut self, vk: winit::event::VirtualKeyCode) {
        if (vk as u32) < 128 {
            self.low_mask |= 1 << (vk as u32);
        } else {
            self.high_mask |= 1 << ((vk as u32) - 128);
        }
    }

    pub fn keyup(&mut self, vk: winit::event::VirtualKeyCode) {
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

/// A user-end trait which enables rendering and response to key presses
pub trait Lepton: 'static {
    /// Respond to a key press. Returns true if the program is to exit.
    fn keydown(&mut self, _keycode: winit::event::VirtualKeyCode) -> bool {false}

    /// Respond to a key release. Returns true if the program is to exit.
    fn keyup(&mut self, _keycode: winit::event::VirtualKeyCode) -> bool {false}

    /// Respond to mouse motion. True if the mouse pointer is to be reset to the center.
    fn mouse_motion(&mut self, _delta: (f64, f64)) -> bool {false}

    /// Determine which pattern to use for drawing
    fn get_pattern(&mut self) -> &mut dyn PatternTrait;

    /// Update all the objects
    fn update(&mut self, delta_time: f32);
}

