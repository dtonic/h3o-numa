use criterion::Criterion;
use h3on::BaseCell;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("res0CellCount");

    group.bench_function("h3on", |b| b.iter(BaseCell::count));
    group.bench_function("h3", |b| {
        b.iter(|| unsafe { h3ron_h3_sys::res0CellCount() })
    });

    group.finish();
}
