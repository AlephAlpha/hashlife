use ca_formats::rle::RLE;
use hashlife::World;
use std::time::Instant;

fn main() {
    let pattern = RLE::new(include_str!("../patterns/c4-diag-switch-engines.rle"));
    let mut world = World::default();
    world.set_step(16);
    for cell in pattern {
        let (x, y) = cell.unwrap();
        world.set_cell(x as i64, y as i64, true);
    }
    println!(
        "Generation: {:?}\tPopulation: {:?}",
        world.get_generation(),
        world.population()
    );
    for _ in 0..16 {
        let now = Instant::now();
        world.step();
        println!(
            "Generation: {:?}\tPopulation: {:?}\tTime: {:?}",
            world.get_generation(),
            world.population(),
            now.elapsed()
        );
    }
}
