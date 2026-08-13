use criterion::{Criterion, criterion_group, criterion_main};

fn reconcile(c: &mut Criterion) {
    c.bench_function("reconcile unchanged tree", |b| {
        b.iter(|| acorn::reconcile("fixtures/workspace"));
    });
}

criterion_group!(benches, reconcile);
criterion_main!(benches);
