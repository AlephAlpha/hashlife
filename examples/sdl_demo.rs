//! Usage:
//!
//! * Enter: Start / stop running
//! * Space: Next step
//! * W / A / S / D / Arrow keys: Move
//! * + / =: Faster
//! * -: Slower
//! * Esc: Quit

use ca_formats::rle::RLE;
use hashlife::World;
use sdl2::{
    self,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::Color,
};
use std::{error::Error, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let pattern = RLE::new(include_str!("../patterns/c4-diag-switch-engines.rle"));
    let mut world = World::default();
    for cell in pattern {
        let (y, x) = cell.unwrap();
        world.set_cell(x as i64, y as i64, true);
    }

    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let window = video_subsystem
        .window("HashLife", 800, 600)
        .resizable()
        .build()?;
    let mut canvas = window.into_canvas().build()?;

    canvas.set_draw_color(Color::WHITE);
    canvas.clear();
    canvas.present();

    let mut is_running = false;

    let mut width = canvas.viewport().width() as i32;
    let mut height = canvas.viewport().height() as i32;
    let mut left = -width / 2;
    let mut top = -height / 2;

    'mainloop: loop {
        for event in sdl.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {
                    left -= x / 2 - width / 2;
                    top -= y / 2 - height / 2;
                    width = x;
                    height = y;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    is_running = false;
                    world.step();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    is_running ^= true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    left -= 10;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    left += 10;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    top -= 10;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    top += 10;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Plus),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Equals),
                    ..
                } => {
                    let step = world.get_step();
                    if step < std::u8::MAX {
                        world.set_step(step + 1);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    let step = world.get_step();
                    if step > 0 {
                        world.set_step(step - 1);
                    }
                }
                _ => {}
            }
        }

        if is_running {
            world.step();
        }

        canvas.set_draw_color(Color::WHITE);
        canvas.clear();
        canvas.set_draw_color(Color::BLACK);

        world.for_living_cells(
            (
                left as i64,
                (left + width) as i64,
                top as i64,
                (top + height) as i64,
            ),
            |x, y| {
                canvas
                    .draw_point((x as i32 - left, y as i32 - top))
                    .unwrap();
            },
        );

        canvas.present();

        sleep(Duration::from_secs(1).div_f32(60.));
    }

    Ok(())
}
