extern crate rustboy;
extern crate sdl2;

mod audio;
mod display;
mod input;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::thread;

fn main() -> Result< (), String >
{
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
        .window("Rustboy - GameBoy Emulator", 100 as u32, 100 as u32)
        .opengl()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;

    'running: loop
    {
        // Handle Events
        for event in event_pump.poll_iter()
        {
            match event
            {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                _ => { continue; }
            }
        }

        // Render
        canvas.clear();
        
        canvas.present();

        // Sleep main thread to avoid overloading CPU
        thread::sleep(::std::time::Duration::from_millis(1));
    }

    Ok(())
}
