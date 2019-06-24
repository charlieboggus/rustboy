extern crate rustboy;
#[macro_use]
extern crate glium;

use glium::{ glutin, Surface, VertexBuffer, index::{ IndexBuffer, PrimitiveType } };
use rustboy::gb::*;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn main()
{
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
        .with_title("Rustboy - GameBoy Emulator");

    // Create context builder. We're using OpenGL 3.3 Core Profile
    let cb = glium::glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
        .with_gl_profile(glutin::GlProfile::Core)
        .with_vsync(true);

    // Create the display
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // Create vertex and index buffers
    let (vertex_buf, index_buf) = {

        #[derive(Clone, Copy)]
        struct Vertex
        {
            pos: [f32; 2],  // Position
            col: [f32; 4],  // Color
            tex: [f32; 2]   // Texture Coords
        }
        implement_vertex!(Vertex, pos, tex, col);

        let vertices = vec![
            Vertex { pos: [-1.0, 1.0], col: [1.0, 0.0, 0.0, 1.0], tex: [0.0, 0.0] },        // Top-Left
            Vertex { pos: [1.0, 1.0], col: [0.0, 1.0, 0.0, 1.0], tex: [1.0, 0.0] },         // Top-Right
            Vertex { pos: [1.0, -1.0], col: [0.0, 0.0, 1.0, 1.0], tex: [1.0, 1.0] },        // Bottom-Right
            Vertex { pos: [-1.0, -1.0], col: [1.0, 1.0, 1.0, 1.0], tex: [0.0, 1.0] },       // Bottom-Left
        ];

        let vb: VertexBuffer< Vertex > = 
            VertexBuffer::new(&display, &vertices).unwrap();
        
        let ib = IndexBuffer::new(&display, PrimitiveType::TriangleStrip, 
            &[1 as u16, 2, 0, 3]).unwrap();

        (vb, ib)
    };

    // Create the shader program
    let program = program!(&display, 330 => { 
        vertex: 
        "
            #version 330 core

            in vec2 pos;
            in vec4 col;
            in vec2 tex;
            out vec4 frag_col;
            out vec2 tex_coords;

            void main()
            {
                frag_col = col;
                tex_coords = tex;
                gl_Position = vec4(pos, 0.0, 1.0);
            }
        ", 

        fragment: 
        "
            #version 330 core

            in vec4 frag_col;
            in vec2 tex_coords;
            out vec4 out_col;
            uniform sampler2D tex;

            void main()
            {
                out_col = texture(tex, tex_coords);
            }
        " 
    }).unwrap();

    // Create GameBoy instance
    let mut gb = Gameboy::new(Path::new("ROMs/Tetris.gb"));

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
                                    glutin::ElementState::Released =>gb.key_up(Button::B)
                                }
                            }

                            if let Some(glutin::VirtualKeyCode::Up) = input.virtual_keycode
                            {
                                match input.state
                                {
                                    glutin::ElementState::Pressed => gb.key_down(Button::Up),
                                    glutin::ElementState::Released =>gb.key_up(Button::Up)
                                }
                            }

                            if let Some(glutin::VirtualKeyCode::Down) = input.virtual_keycode
                            {
                                match input.state
                                {
                                    glutin::ElementState::Pressed => gb.key_down(Button::Down),
                                    glutin::ElementState::Released =>gb.key_up(Button::Down)
                                }
                            }

                            if let Some(glutin::VirtualKeyCode::Left) = input.virtual_keycode
                            {
                                match input.state
                                {
                                    glutin::ElementState::Pressed => gb.key_down(Button::Left),
                                    glutin::ElementState::Released =>gb.key_up(Button::Left)
                                }
                            }

                            if let Some(glutin::VirtualKeyCode::Right) = input.virtual_keycode
                            {
                                match input.state
                                {
                                    glutin::ElementState::Pressed => gb.key_down(Button::Right),
                                    glutin::ElementState::Released =>gb.key_up(Button::Right)
                                }
                            }

                            if let Some(glutin::VirtualKeyCode::O) = input.virtual_keycode
                            {
                                match input.state
                                {
                                    glutin::ElementState::Pressed => gb.key_down(Button::Start),
                                    glutin::ElementState::Released =>gb.key_up(Button::Start)
                                }
                            }

                            if let Some(glutin::VirtualKeyCode::P) = input.virtual_keycode
                            {
                                match input.state
                                {
                                    glutin::ElementState::Pressed => gb.key_down(Button::Select),
                                    glutin::ElementState::Released =>gb.key_up(Button::Select)
                                }
                            }
                        },
                        _ => ()
                    }
                },

                _ => ()
            }
        });

        // Execute GameBoy cycle
        gb.run();

        // Create texture from GameBoy GPU image data
        let image = glium::texture::RawImage2d::from_raw_rgba(gb.get_image_data().to_vec(), (DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32));
        let opengl_tex = glium::texture::texture2d::Texture2d::new(&display, image).unwrap();

        // Create uniforms
        let uniforms = uniform! { tex: &opengl_tex };

        // Draw
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.draw(&vertex_buf, &index_buf, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        // Sleep main thread to avoid overloading CPU
        thread::sleep(Duration::from_millis(10));
    }
}
