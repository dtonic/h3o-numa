use criterion::{black_box, Bencher, Criterion};
use h3on::CellIndex;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("areNeighborCells_h3o_vs_h3on");

    // Same parent center - 가장 빠른 경우
    let (origin, index) = (0x0890153a1017ffff, 0x890153a1003ffff);
    group.bench_function("h3on/SameParentCenter", |b| {
        bench_h3on(b, index, origin)
    });
    group.bench_function("h3o/SameParentCenter", |b| bench_h3o(b, index, origin));

    // Same parent other - 같은 부모, 다른 위치
    let (origin, index) = (0x0890153a1017ffff, 0x0890153a1013ffff);
    group.bench_function("h3on/SameParentOther", |b| {
        bench_h3on(b, index, origin)
    });
    group.bench_function("h3o/SameParentOther", |b| bench_h3o(b, index, origin));

    // Different parent - 다른 부모, 빠른 구현 사용
    let (origin, index) = (0x0890153a1017ffff, 0x0890153a10bbffff);
    group.bench_function("h3on/DifferentParent", |b| {
        bench_h3on(b, index, origin)
    });
    group.bench_function("h3o/DifferentParent", |b| bench_h3o(b, index, origin));

    // Different parent fallback - 다른 부모, 느린 구현 사용
    let (origin, index) = (0x08908000001bffff, 0x08908000000fffff);
    group.bench_function("h3on/DifferentParentFallback", |b| {
        bench_h3on(b, index, origin)
    });
    group.bench_function("h3o/DifferentParentFallback", |b| {
        bench_h3o(b, index, origin)
    });

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3on(b: &mut Bencher<'_>, index: u64, origin: u64) {
    let origin = CellIndex::try_from(origin).expect("origin");
    let index = CellIndex::try_from(index).expect("index");
    b.iter(|| black_box(origin).is_neighbor_with(black_box(index)))
}

fn bench_h3o(b: &mut Bencher<'_>, index: u64, origin: u64) {
    let origin = h3o::CellIndex::try_from(origin).expect("origin");
    let index = h3o::CellIndex::try_from(index).expect("index");
    b.iter(|| black_box(origin).is_neighbor_with(black_box(index)))
}

