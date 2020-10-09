use ca_formats::rle::RLE;
use hashlife::World;
use sdl2::{
    self,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::Color,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let pattern = RLE::new(include_str!("../patterns/c4-diag-switch-engines.rle"));
    let mut world = World::default();
    world.set_step(6);
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

    let mut width = canvas.viewport().width() as i32;
    let mut height = canvas.viewport().height() as i32;

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
                    width = x;
                    height = y;
                }
                _ => {}
            }
        }

        for x in 0..width {
            for y in 0..height {
                let cell = world.get_cell((x - width / 2) as i64, (y - height / 2) as i64);
                if cell {
                    canvas.set_draw_color(Color::BLACK);
                } else {
                    canvas.set_draw_color(Color::WHITE);
                }
                canvas.draw_point((x, y))?;
            }
        }
        canvas.present();
        world.step();
    }

    Ok(())
}
