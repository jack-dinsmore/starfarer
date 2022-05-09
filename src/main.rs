mod graphics;
mod util;

use glow::*;
use graphics::shaders::{ShaderManager, Shader};
use graphics::models::Model;

//use glow_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text, Region};
use cgmath::{Matrix4, Point3, Vector3, PerspectiveFov, Rad};


fn main() {
    unsafe {
        // Create a context from a sdl2 window
        let (gl, window, mut events_loop, _context) = create_sdl2_context();

        // Create a shader program from source
        let shader_manager = ShaderManager::new(&gl);

        let view = Matrix4::look_at_rh(Point3::new(5.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0));

        let projection = Matrix4::from(PerspectiveFov{ fovy: Rad(0.9), aspect: 1.2, near: 0.01, far: 100.0});

        let mvp = projection * view;

        // Upload uniforms
        shader_manager.set_uniforms(Shader::Object(&mvp));

        // Prepare glyph_brush
        //let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf")).expect("Could not open font file");
        //let mut glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build(&gl);

        // Create a vertex buffer and vertex array object
        let model = Model::new(&gl, include_str!("assets/cube/cube.obj"));

        gl.enable(glow::FRAMEBUFFER_SRGB);
        gl.enable(glow::BLEND);
        gl.disable(glow::CULL_FACE);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        gl.enable(glow::DEPTH_TEST);
        gl.depth_func(glow::LESS);
        gl.clear_color(0.1, 0.2, 0.3, 0.0);

        'render: loop {
            for event in events_loop.poll_iter() {
                match event {
                    sdl2::event::Event::Quit{..} => break 'render,
                    _ => ()
                }
            }

            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            
            // Queue text to be drawn
            /*glyph_brush.queue(Section {
                screen_position: (30.0, 30.0),
                bounds: (1024.0, 769.0),
                text: vec![Text::default()
                    .with_text("Hello glow_glyph!")
                    .with_color([0.0, 0.0, 0.0, 1.0])
                    .with_scale(40.0)],
                ..Section::default()
            });

            // Draw text
            glyph_brush.draw_queued(&gl, 1024, 769).expect("Draw queued");*/
            shader_manager.set_uniforms(Shader::Object(&mvp));
            shader_manager.load_object();

            model.draw(&gl);

            window.gl_swap_window();
        }
        model.clean(&gl);
    }
}

unsafe fn create_sdl2_context() -> (
    glow::Context,
    sdl2::video::Window,
    sdl2::EventPump,
    sdl2::video::GLContext,
) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 0);
    let window = video
        .window("Hello triangle!", 1024, 769)
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
    let event_loop = sdl.event_pump().unwrap();

    (gl, window, event_loop, gl_context)
}