use flate2::read::GzDecoder;
use hashlife::{
    ca_formats::{macrocell::Macrocell, rle::Rle},
    World,
};
use sdl2::{
    self,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    mouse::MouseState,
    pixels::Color,
    rect::Rect,
    render::Canvas,
    video::Window,
    Sdl,
};
use std::{
    env::args,
    error::Error,
    fs::File,
    thread::sleep,
    time::{Duration, Instant},
};

const SCALE_OFFSET: u8 = 4;
const MAX_SCALE: u8 = 63 + SCALE_OFFSET;
const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);

fn make_world() -> Result<World, Box<dyn Error>> {
    let arg = args().nth(1);
    if let Some(path) = arg {
        let file = File::open(&path)?;
        if path.ends_with(".gz") {
            let unzipped = GzDecoder::new(file);
            if path.ends_with(".mc.gz") {
                let macrocell = Macrocell::new_from_file(unzipped)?;
                Ok(World::from_macrocell(macrocell)?)
            } else {
                let rle = Rle::new_from_file(unzipped)?;
                Ok(World::from_rle(rle)?)
            }
        } else if path.ends_with(".mc") {
            let macrocell = Macrocell::new_from_file(file)?;
            Ok(World::from_macrocell(macrocell)?)
        } else {
            let rle = Rle::new_from_file(file)?;
            Ok(World::from_rle(rle)?)
        }
    } else {
        let rle = Rle::new("b2o$2o$bo!")?;
        Ok(World::from_rle(rle)?)
    }
}

struct App {
    world: World,
    sdl: Sdl,
    canvas: Canvas<Window>,
    is_running: bool,
    need_update: bool,
    width: i64,
    height: i64,
    left: i64,
    top: i64,
    scale: u8,
}

impl App {
    fn new(world: World) -> Result<Self, Box<dyn Error>> {
        let sdl = sdl2::init()?;
        let video_subsystem = sdl.video()?;
        let window = video_subsystem
            .window("HashLife", 800, 600)
            .resizable()
            .build()?;
        let canvas = window.into_canvas().build()?;

        let is_running = false;
        let need_update = true;

        let width = canvas.viewport().width() as i64;
        let height = canvas.viewport().height() as i64;
        let left = -width / 2;
        let top = -height / 2;
        let scale = 0;
        let mut app = Self {
            world,
            sdl,
            canvas,
            is_running,
            need_update,
            width,
            height,
            left,
            top,
            scale,
        };

        app.fit();
        app.update_canvas();

        Ok(app)
    }

    fn fit(&mut self) {
        if let Some(bound) = self.world.bound() {
            let pattern_width = bound.1 - bound.0;
            let pattern_height = bound.3 - bound.2;
            let center = ((bound.0 + bound.1) / 2, (bound.2 + bound.3) / 2);
            let scale_diff = (pattern_width as f64 / self.width as f64)
                .max(pattern_height as f64 / self.height as f64)
                .log2()
                .ceil() as i32;
            self.scale = (SCALE_OFFSET as i32 + scale_diff).clamp(0, MAX_SCALE as i32) as u8;
            if self.scale >= SCALE_OFFSET {
                self.left = (center.0 >> (self.scale - SCALE_OFFSET)) - self.width / 2;
                self.top = (center.1 >> (self.scale - SCALE_OFFSET)) - self.height / 2;
            } else {
                self.left = (center.0 << (SCALE_OFFSET - self.scale)) - self.width / 2;
                self.top = (center.1 << (SCALE_OFFSET - self.scale)) - self.height / 2;
            }
        } else {
            self.scale = 0;
            self.left = -self.width / 2;
            self.top = -self.height / 2;
        }
        self.need_update = true;
    }

    fn window_resize(&mut self, x: i64, y: i64) {
        self.left -= x / 2 - self.width / 2;
        self.top -= y / 2 - self.height / 2;
        self.width = x;
        self.height = y;
        self.need_update = true;
    }

    fn one_step(&mut self) {
        self.is_running = false;
        self.world.step();
        self.need_update = true;
    }

    fn faster(&mut self) {
        let step = self.world.get_step();
        if step < std::u8::MAX {
            self.world.set_step(step + 1);
            self.need_update = true;
        }
    }

    fn slower(&mut self) {
        let step = self.world.get_step();
        if step > 0 {
            self.world.set_step(step - 1);
            self.need_update = true;
        }
    }

    fn move_canvas(&mut self, x: i64, y: i64) {
        self.left -= x;
        self.top -= y;
        self.need_update = true;
    }

    fn rescale(&mut self, y: i32, mouse_state: MouseState) {
        let (mouse_x, mouse_y) = (mouse_state.x() as i64, mouse_state.y() as i64);
        let new_scale = (self.scale as i32 - y).clamp(0, MAX_SCALE as i32) as u8;
        if new_scale > self.scale {
            self.left = ((self.left + mouse_x) >> (new_scale - self.scale)) - mouse_x;
            self.top = ((self.top + mouse_y) >> (new_scale - self.scale)) - mouse_y;
        } else {
            self.left = ((self.left + mouse_x) << (self.scale - new_scale)) - mouse_x;
            self.top = ((self.top + mouse_y) << (self.scale - new_scale)) - mouse_y;
        }
        self.scale = new_scale;
        self.need_update = true;
    }

    fn update_canvas(&mut self) {
        let canvas = &mut self.canvas;
        let left = self.left;
        let top = self.top;

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.set_draw_color(Color::WHITE);

        let bound = (left, left + self.width, top, top + self.height);

        if self.scale >= SCALE_OFFSET {
            self.world
                .for_nodes(self.scale - SCALE_OFFSET, bound, |x, y| {
                    canvas
                        .draw_point(((x - left) as i32, (y - top) as i32))
                        .unwrap();
                });
        } else {
            let neg_scale = SCALE_OFFSET - self.scale;
            let bound = (
                (bound.0 >> neg_scale) - 1,
                (bound.1 >> neg_scale) + 1,
                (bound.2 >> neg_scale) - 1,
                (bound.3 >> neg_scale) + 1,
            );

            self.world.for_nodes(0, bound, |x, y| {
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

    fn log(&self, now: &Instant) {
        eprintln!(
            "{}\tGen: {:?}\tStep: 2^{:?}\tPop: {:?}\tScale: 1:2^{:?}\tFps: {:?}",
            if self.is_running { "Running" } else { "Paused" },
            self.world.get_generation(),
            self.world.get_step(),
            self.world.population(),
            self.scale as i32 - SCALE_OFFSET as i32,
            1.0 / now.elapsed().as_secs_f32(),
        );
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut now = Instant::now();

        'mainloop: loop {
            let mut events = self.sdl.event_pump()?;
            let mouse_state = events.mouse_state();

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
                    } => self.window_resize(x as i64, y as i64),
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        ..
                    } => self.one_step(),
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => self.is_running ^= true,
                    Event::KeyDown {
                        keycode: Some(Keycode::Plus | Keycode::Equals),
                        ..
                    } => self.faster(),
                    Event::KeyDown {
                        keycode: Some(Keycode::Minus),
                        ..
                    } => self.slower(),
                    Event::KeyDown {
                        keycode: Some(Keycode::A | Keycode::Left),
                        ..
                    } => self.move_canvas(10, 0),
                    Event::KeyDown {
                        keycode: Some(Keycode::D | Keycode::Right),
                        ..
                    } => self.move_canvas(-10, 0),
                    Event::KeyDown {
                        keycode: Some(Keycode::W | Keycode::Up),
                        ..
                    } => self.move_canvas(0, 10),
                    Event::KeyDown {
                        keycode: Some(Keycode::S | Keycode::Down),
                        ..
                    } => self.move_canvas(0, -10),
                    Event::MouseMotion { xrel, yrel, .. } => {
                        if mouse_state.left() {
                            self.move_canvas(xrel as i64, yrel as i64);
                        }
                    }
                    Event::MouseWheel { y, .. } => self.rescale(y, mouse_state),
                    Event::KeyDown {
                        keycode: Some(Keycode::F),
                        ..
                    } => self.fit(),
                    _ => {}
                }
            }

            if self.is_running {
                self.world.step();
                self.need_update = true;
            }

            if self.need_update {
                self.update_canvas();
            }

            let time_taken = now.elapsed();
            if FRAME_TIME > time_taken {
                sleep(FRAME_TIME - time_taken);
            }

            if self.need_update {
                self.log(&now);
            }
            self.need_update = false;
            now = Instant::now();
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let world = make_world()?;

    let mut app = App::new(world)?;
    app.run()
}
