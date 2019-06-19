extern crate rustboy;
#[macro_use]
extern crate glium;

use glium::{ glutin, Surface };
use rustboy::gb::*;
use std::path::Path;

fn main()
{
    // Create the GB
    let mut gb = Gameboy::new(Path::new(""));

    // Display scaling stuff
    let ratio = 1 + (DISPLAY_WIDTH / 10);
    let width = DISPLAY_WIDTH + 10 * ratio;
    let height = DISPLAY_HEIGHT + 9 * ratio;

    // Create event loop
    let mut event_loop = glutin::EventsLoop::new();

    // Create window builder
    let wb = glium::glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(width as f64, height as f64))
        .with_resizable(false)
        .with_title("Rustboy - GameBoy Emulator - ".to_string() + &*gb.title);

    // Create context builder. We're using the latest version of OpenGL Core
    let cb = glium::glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Latest)
        .with_gl_profile(glutin::GlProfile::Core)
        .with_vsync(true);

    // Create the display
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // Finally, power on the GB before entering application loop
    gb.power_on();

    // Primary application loop
    let mut closed = false;
    while !closed
    {
        // Event loop
        event_loop.poll_events(|e| 
        {
            match e
            {
                glutin::Event::WindowEvent { event, .. } => 
                {
                    match event 
                    {
                        // Window close event
                        glutin::WindowEvent::CloseRequested => closed = true,

                        // Keyboard input event
                        glutin::WindowEvent::KeyboardInput { input, .. } => 
                        {
                            if let Some(glutin::VirtualKeyCode::Z) = input.virtual_keycode
                            {
                                match input.state
                                {
                                    glutin::ElementState::Pressed => gb.key_down(Button::A),
                                    glutin::ElementState::Released => gb.key_up(Button::A)
                                }
                            }

                            if let Some(glutin::VirtualKeyCode::X) = input.virtual_keycode
                            {
                                match input.state
                                {
                                    glutin::ElementState::Pressed => gb.key_down(Button::B),
                                    glutin::ElementState::Released => gb.key_up(Button::B)
                                }
                            }

                            // TODO: rest of the controls
                        },
                        _ => ()
                    }
                },

                _ => ()
            }
        });

        // Draw
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        // TODO: draw here
        target.finish().unwrap();
    }
}
