use ca_formats::rle::Rle;
use hashlife::World;

fn run_pattern(pattern: &str, rule_string: &str, step_log2: u8) -> u64 {
    let mut world = World::new_with_step(rule_string.parse().unwrap(), step_log2);
    for cell in Rle::new(pattern).unwrap() {
        let (x, y) = cell.unwrap().position;
        world.set_cell(x, y, true);
    }
    world.step();
    world.population()
}

#[test]
fn c4_diag_switch_engines() {
    assert_eq!(
        run_pattern(
            include_str!("../patterns/c4-diag-switch-engines.rle"),
            "B3/S23",
            16
        ),
        361207
    );
}

#[test]
fn switch_engine_breeder() {
    assert_eq!(
        run_pattern(
            include_str!("../patterns/switch-engine-breeder.rle"),
            "B3/S23",
            20
        ),
        764025216
    );
}

#[test]
fn zigzag_wickstretcher() {
    assert_eq!(
        run_pattern(
            include_str!("../patterns/zigzag-wickstretcher.rle"),
            "B3/S23",
            20
        ),
        604779
    );
}

#[test]
fn sierpinski_builder() {
    assert_eq!(
        run_pattern(
            include_str!("../patterns/Sierpinski-builder.rle"),
            "B3/S23-a4ei6",
            20
        ),
        129274688
    );
}
