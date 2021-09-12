use criterion::{black_box, criterion_group, criterion_main, Criterion};
use voila::run;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Voila, [benchmark name here]", |b| {
        b.iter(|| {
            run(
                black_box("@name == @name { print(@name.file); print(@a, @parent) }"),
                black_box(std::path::PathBuf::from(env!("HOME"))),
                black_box(true),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
