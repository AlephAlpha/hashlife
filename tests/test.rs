use ca_formats::rle::RLE;
use hashlife::World;

fn run_pattern(pattern: &str, rule_string: &str, step_log2: u8) -> u64 {
    let mut world = World::new(rule_string.parse().unwrap(), step_log2);
    for cell in RLE::new(pattern) {
        let (x, y) = cell.unwrap();
        world.set_cell(x as i64, y as i64, true);
    }
    world.step();
    world.population()
}

#[test]
fn c4_diag_switch_engines() {
    assert_eq!(
        run_pattern(include_str!("c4-diag-switch-engines.rle"), "B3/S23", 16),
        361207
    );
}

#[test]
fn switch_engine_breeder() {
    assert_eq!(
        run_pattern(include_str!("switch-engine-breeder.rle"), "B3/S23", 20),
        764025216
    );
}

#[test]
fn zigzag_wickstretcher() {
    assert_eq!(
        run_pattern(include_str!("zigzag-wickstretcher.rle"), "B3/S23", 20),
        604779
    );
}

#[test]
fn sierpinski_builder() {
    assert_eq!(
        run_pattern(include_str!("Sierpinski-builder.rle"), "B3/S23-a4ei6", 20),
        129274688
    );
}
