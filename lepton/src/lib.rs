mod vk_core;

use vk_core::VulkanData;
use vk_core::window::VulkanApp;

use vk_core::fps_limiter::FPSLimiter;
use winit::{event::{Event, WindowEvent, KeyboardInput}, event_loop::{EventLoop, ControlFlow}};

/// A `lepton`-end object which wraps all the Vulkan graphics objects and is responsible for creating the game window and graphics pipeline and rendering and loading models.
pub struct Graphics {
    vulkan_data: VulkanData,
}

impl Graphics {
    /// Initialize the Vulkan pipeline and open the window
    pub fn new(control: &Control) -> Self {
        let vulkan_data = VulkanData::new(&control.event_loop);

        Graphics { vulkan_data }
    }

    /// Close the window and disable the Vulkan pipeline
    fn terminate(&self) {
        self.vulkan_data.wait_device_idle();
    }

    /// Submit redraw queue in Vulkan pipeline
    fn request_redraw(&self) {
        self.vulkan_data.window_ref().request_redraw();
    }
}

pub struct Control {
    event_loop: EventLoop<()>
}

impl Control{
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

                    renderer.draw_frame(delta_time);
                    graphics.vulkan_data.draw_frame(delta_time);
                    
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
    fn draw_frame(&mut self, delta_time: f32);
}

