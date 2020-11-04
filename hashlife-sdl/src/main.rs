use hashlife::{
    ca_formats::{macrocell::Macrocell, rle::Rle},
    World,
};
use sdl2::{
    self,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
};
use std::{
    env::args,
    error::Error,
    fs::File,
    thread::sleep,
    time::{Duration, Instant},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut world;
    let arg = args().skip(1).next();
    if let Some(path) = arg {
        let file = File::open(&path)?;
        if path.ends_with(".mc") {
            let macrocell = Macrocell::new_from_file(file)?;
            world = World::from_macrocell(macrocell)?;
        } else {
            let rle = Rle::new_from_file(file)?;
            world = World::from_rle(rle)?;
        }
    } else {
        let macrocell = Macrocell::new(include_str!("../../patterns/metapixel-galaxy.mc"))?;
        world = World::from_macrocell(macrocell)?;
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

    const SCALE_OFFSET: u8 = 4;
    let mut width = canvas.viewport().width() as i64;
    let mut height = canvas.viewport().height() as i64;
    let mut left;
    let mut top;
    let mut scale;

    if let Some(bound) = world.bound() {
        let pattern_width = bound.1 - bound.0;
        let pattern_height = bound.3 - bound.2;
        let center = ((bound.0 + bound.1) / 2, (bound.2 + bound.3) / 2);
        let scale_diff = (pattern_width as f64 / width as f64)
            .max(pattern_height as f64 / height as f64)
            .log2()
            .ceil() as i32;
        scale = (SCALE_OFFSET as i32 + scale_diff)
            .max(0)
            .min(63 + SCALE_OFFSET as i32) as u8;
        if scale >= SCALE_OFFSET {
            left = (center.0 >> (scale - SCALE_OFFSET)) - width / 2;
            top = (center.1 >> (scale - SCALE_OFFSET)) - height / 2;
        } else {
            left = (center.0 << (SCALE_OFFSET - scale)) - width / 2;
            top = (center.1 << (SCALE_OFFSET - scale)) - height / 2;
        }
    } else {
        scale = 0;
        left = -width / 2;
        top = -height / 2;
    }

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
                    let new_scale = (scale as i32 - y).max(0).min(63 + SCALE_OFFSET as i32) as u8;
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
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => {
                    if let Some(bound) = world.bound() {
                        let pattern_width = bound.1 - bound.0;
                        let pattern_height = bound.3 - bound.2;
                        let center = ((bound.0 + bound.1) / 2, (bound.2 + bound.3) / 2);
                        let scale_diff = (pattern_width as f64 / width as f64)
                            .max(pattern_height as f64 / height as f64)
                            .log2()
                            .ceil() as i32;
                        scale = (SCALE_OFFSET as i32 + scale_diff)
                            .max(0)
                            .min(63 + SCALE_OFFSET as i32) as u8;
                        if scale >= SCALE_OFFSET {
                            left = (center.0 >> (scale - SCALE_OFFSET)) - width / 2;
                            top = (center.1 >> (scale - SCALE_OFFSET)) - height / 2;
                        } else {
                            left = (center.0 << (SCALE_OFFSET - scale)) - width / 2;
                            top = (center.1 << (SCALE_OFFSET - scale)) - height / 2;
                        }
                    } else {
                        scale = 0;
                        left = -width / 2;
                        top = -height / 2;
                    }
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

            let bound = (
                left as i64,
                (left + width) as i64,
                top as i64,
                (top + height) as i64,
            );

            if scale >= SCALE_OFFSET {
                world.for_nodes(scale - SCALE_OFFSET, bound, |x, y| {
                    canvas
                        .draw_point(((x - left) as i32, (y - top) as i32))
                        .unwrap();
                });
            } else {
                let neg_scale = SCALE_OFFSET - scale;
                let bound = (
                    (bound.0 >> neg_scale) - 1,
                    (bound.1 >> neg_scale) + 1,
                    (bound.2 >> neg_scale) - 1,
                    (bound.3 >> neg_scale) + 1,
                );

                world.for_nodes(0, bound, |x, y| {
                    canvas
                        .fill_rect(Rect::new(
                            ((x << neg_scale) - left) as i32,
                            ((y << neg_scale) - top) as i32,
                            1 << neg_scale,
                            1 << neg_scale,
                        ))
                        .unwrap();
                });
            }

            canvas.present();
        }

        let time_taken = now.elapsed();
        if FRAME_TIME > time_taken {
            sleep(FRAME_TIME - time_taken);
        }

        if need_update {
            eprintln!(
                "{}\tGen: {:?}\tStep: 2^{:?}\tPop: {:?}\tScale: 1:2^{:?}\tFps: {:?}",
                if is_running { "Running" } else { "Paused" },
                world.get_generation(),
                world.get_step(),
                world.population(),
                scale as i32 - SCALE_OFFSET as i32,
                1.0 / now.elapsed().as_secs_f32(),
            );
        }
        need_update = false;
        now = Instant::now();
    }

    Ok(())
}
