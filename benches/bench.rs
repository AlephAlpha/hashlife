use ca_formats::{macrocell::Macrocell, rle::Rle};
use criterion::{criterion_group, criterion_main, Criterion};
use hashlife::World;
use std::time::Duration;

fn run_pattern(pattern: &str, step_log2: u8, steps: u32) {
    let rle = Rle::new(pattern).unwrap();
    let mut world = World::from_rle(rle).unwrap();
    world.set_step(step_log2);
    for _ in 0..steps {
        world.step();
    }
}

fn test_gc(with_gc: bool) {
    let macrocell = Macrocell::new(include_str!("../patterns/metapixel-galaxy.mc")).unwrap();
    let mut world = World::from_macrocell(macrocell).unwrap();
    world.set_step(12);
    for _ in 0..16 {
        world.step();
    }
    if with_gc {
        world.garbage_collect();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("All tests");
    group.warm_up_time(Duration::from_secs(10));
    group.measurement_time(Duration::from_secs(60));

    group
        .bench_function("c4-diag-switch-engines", |b| {
            b.iter(|| {
                run_pattern(
                    include_str!("../patterns/c4-diag-switch-engines.rle"),
                    16,
                    16,
                )
            })
        })
        .bench_function("switch-engine-breeder", |b| {
            b.iter(|| {
                run_pattern(
                    include_str!("../patterns/switch-engine-breeder.rle"),
                    20,
                    16,
                )
            })
        })
        .bench_function("zigzag-wickstretcher", |b| {
            b.iter(|| run_pattern(include_str!("../patterns/zigzag-wickstretcher.rle"), 20, 16))
        })
        .bench_function("sierpinski-builder", |b| {
            b.iter(|| run_pattern(include_str!("../patterns/Sierpinski-builder.rle"), 20, 16))
        })
        .bench_function("without_gc", |b| b.iter(|| test_gc(false)))
        .bench_function("with_gc", |b| b.iter(|| test_gc(true)));

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
