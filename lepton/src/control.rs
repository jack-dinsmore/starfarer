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
    pub fn run<K, R>(self, mut graphics: Graphics, mut key_event: Option<K>, mut renderer: R, print_fps: bool) -> ! 
    where K: KeyEvent + 'static, R: Renderer + 'static {
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
                                    if let Some(ke) = &mut key_event {
                                        if ke.keydown(virtual_keycode, state) {
                                            graphics.terminate();
                                            *control_flow = ControlFlow::Exit
                                        }
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

                    renderer.update(delta_time);
                    graphics.draw_frame(renderer.get_pattern());
                    
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

/// A user-end trait which enables response to key presses
pub trait KeyEvent {
    /// Respond to a key press. Returns true if the program is to exit.
    fn keydown(&mut self, keycode: Option<winit::event::VirtualKeyCode>, element_state: winit::event::ElementState) -> bool;
}

/// A user-end trait which enables rendering.
pub trait Renderer {
    /// Draw one frame
    fn get_pattern(&self) -> &dyn PatternTrait;
    fn update(&mut self, delta_time: f32);
}

