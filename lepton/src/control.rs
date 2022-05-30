use winit::{event::{Event, WindowEvent, KeyboardInput}, event_loop::{EventLoop, ControlFlow}};
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
                | Event::WindowEvent { event, ..} => {

                    match event {
                        | WindowEvent::CloseRequested => {
                            graphics.terminate();
                            *control_flow = ControlFlow::Exit
                        },
                        | WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                | KeyboardInput { virtual_keycode, state, .. } => {
                                    if lepton.keydown(virtual_keycode, state) {
                                        graphics.terminate();
                                        *control_flow = ControlFlow::Exit
                                    }
                                },
                            }
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

/// A user-end trait which enables rendering and response to key presses
pub trait Lepton: 'static {
    /// Respond to a key press. Returns true if the program is to exit.
    fn keydown(&mut self, _keycode: Option<winit::event::VirtualKeyCode>, _element_state: winit::event::ElementState) -> bool {false}

    /// Determine which pattern to use for drawing
    fn get_pattern(&self) -> &dyn PatternTrait;

    // Update all the objects
    fn update(&mut self, delta_time: f32);
}

