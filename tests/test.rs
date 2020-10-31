use ca_formats::{macrocell::Macrocell, rle::Rle};
use hashlife::World;

fn run_rle(pattern: &str, step_log2: u8) -> u64 {
    let rle = Rle::new(pattern).unwrap();
    let mut world = World::from_rle(rle).unwrap();
    world.set_step(step_log2);
    world.step();
    world.population()
}

fn run_macrocell(pattern: &str, step_log2: u8) -> u64 {
    let macrocell = Macrocell::new(pattern).unwrap();
    let mut world = World::from_macrocell(macrocell).unwrap();
    world.set_step(step_log2);
    world.step();
    world.population()
}

#[test]
fn c4_diag_switch_engines() {
    assert_eq!(
        run_rle(include_str!("../patterns/c4-diag-switch-engines.rle"), 16),
        361207
    );
}

#[test]
fn switch_engine_breeder() {
    assert_eq!(
        run_rle(include_str!("../patterns/switch-engine-breeder.rle"), 20),
        764025216
    );
}

#[test]
fn zigzag_wickstretcher() {
    assert_eq!(
        run_rle(include_str!("../patterns/zigzag-wickstretcher.rle"), 20),
        604779
    );
}

#[test]
fn sierpinski_builder() {
    assert_eq!(
        run_rle(include_str!("../patterns/Sierpinski-builder.rle"), 20),
        129274688
    );
}

#[test]
fn totalperiodic() {
    assert_eq!(
        run_macrocell(include_str!("../patterns/totalperiodic.mc"), 16),
        74390
    );
}

#[test]
fn demonoid_c512_hashlife_friendly() {
    assert_eq!(
        run_macrocell(
            include_str!("../patterns/demonoid-c512-hashlife-friendly.mc"),
            12
        ),
        107005
    );
}
