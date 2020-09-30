use ca_formats::rle::RLE;
use hashlife::World;

fn main() {
    let rule = "B3/S23".parse().unwrap();
    let pattern = RLE::new(include_str!("../patterns/c4-diag-switch-engines.rle"));
    let mut world = World::new_with_step(rule, 16);
    for cell in pattern {
        let (x, y) = cell.unwrap();
        world.set_cell(x as i64, y as i64, true);
    }
    world.step();
    println!("{}", world.population());
}
