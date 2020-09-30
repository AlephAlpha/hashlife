use ca_formats::rle::RLE;
use criterion::{criterion_group, criterion_main, Criterion};
use hashlife::World;
use std::time::Duration;

fn run_pattern(pattern: &str, rule_string: &str, step_log2: u8, steps: u32) {
    let mut world = World::new(rule_string.parse().unwrap(), step_log2);
    for cell in RLE::new(pattern) {
        let (x, y) = cell.unwrap();
        world.set_cell(x as i64, y as i64, true);
    }
    for _ in 0..steps {
        world.step();
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
                    "B3/S23",
                    16,
                    16,
                )
            })
        })
        .bench_function("switch-engine-breeder", |b| {
            b.iter(|| {
                run_pattern(
                    include_str!("../patterns/switch-engine-breeder.rle"),
                    "B3/S23",
                    20,
                    16,
                )
            })
        })
        .bench_function("zigzag-wickstretcher", |b| {
            b.iter(|| {
                run_pattern(
                    include_str!("../patterns/zigzag-wickstretcher.rle"),
                    "B3/S23",
                    20,
                    16,
                )
            })
        })
        .bench_function("sierpinski-builder", |b| {
            b.iter(|| {
                run_pattern(
                    include_str!("../patterns/Sierpinski-builder.rle"),
                    "B3/S23-a4ei6",
                    20,
                    16,
                )
            })
        });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
