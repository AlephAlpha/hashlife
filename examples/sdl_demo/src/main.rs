use ca_formats::rle::RLE;
use hashlife::World;
use sdl2::{
    self,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::Color,
};
use std::{
    error::Error,
    thread::sleep,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn Error>> {
    let pattern = RLE::new(include_str!("../../../patterns/c4-diag-switch-engines.rle"));
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

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();

    let mut is_running = false;
    let mut need_update = true;

    let mut width = canvas.viewport().width() as i64;
    let mut height = canvas.viewport().height() as i64;
    let mut left = -width / 2;
    let mut top = -height / 2;
    let mut scale = 0;

    let mut now = Instant::now();
    const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);

    'mainloop: loop {
        let mut events = sdl.event_pump()?;
        let mouse_state = events.mouse_state();
        let (mouse_x, mouse_y) = (mouse_state.x() as i64, mouse_state.y() as i64);

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::Window {
                    win_event: WindowEvent::SizeChanged(x, y),
                    ..
                } => {
                    left -= x as i64 / 2 - width / 2;
                    top -= y as i64 / 2 - height / 2;
                    width = x as i64;
                    height = y as i64;
                    need_update = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    is_running = false;
                    world.step();
                    need_update = true;
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
                    need_update = true;
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
                    need_update = true;
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
                    need_update = true;
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
                    need_update = true;
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
                Event::MouseMotion {
                    xrel,
                    yrel,
                    mousestate,
                    ..
                } => {
                    if mousestate.left() {
                        left -= xrel as i64;
                        top -= yrel as i64;
                        need_update = true;
                    }
                }
                Event::MouseWheel { y, .. } => {
                    let new_scale = (scale as i32 - y).max(0).min(0xff) as u8;
                    if new_scale > scale {
                        left = ((left + mouse_x) >> (new_scale - scale)) - mouse_x;
                        top = ((top + mouse_y) >> (new_scale - scale)) - mouse_y;
                    } else {
                        left = ((left + mouse_x) << (scale - new_scale)) - mouse_x;
                        top = ((top + mouse_y) << (scale - new_scale)) - mouse_y;
                    }
                    scale = new_scale;
                    need_update = true;
                }
                _ => {}
            }
        }

        if is_running {
            world.step();
            need_update = true;
        }

        if need_update {
            canvas.set_draw_color(Color::BLACK);
            canvas.clear();
            canvas.set_draw_color(Color::WHITE);

            world.for_nodes(
                scale,
                (
                    left as i64,
                    (left + width) as i64,
                    top as i64,
                    (top + height) as i64,
                ),
                |x, y| {
                    canvas
                        .draw_point(((x - left) as i32, (y - top) as i32))
                        .unwrap();
                },
            );

            canvas.present();
        }

        need_update = false;

        let time_taken = now.elapsed();
        if FRAME_TIME > time_taken {
            sleep(FRAME_TIME - time_taken);
        }

        eprintln!(
            "{}\tGen: {:?}\tStep: 2^{:?}\tPop: {:?}\tScale: 1:2^{:?}\tFps: {:?}",
            if is_running { "Running" } else { "Paused" },
            world.get_generation(),
            world.get_step(),
            world.population(),
            scale,
            1.0 / now.elapsed().as_secs_f32(),
        );

        now = Instant::now();
    }

    Ok(())
}
