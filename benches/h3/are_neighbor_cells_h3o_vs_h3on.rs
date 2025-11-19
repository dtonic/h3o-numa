use criterion::{Bencher, Criterion};
use h3on::CellIndex;
use std::hint::black_box;

pub fn bench(c: &mut Criterion) {
    use criterion::BenchmarkId;
    let mut group = c.benchmark_group("areNeighborCells_h3o_vs_h3on");

    let cases = [
        ("SameParentCenter", 0x0890153a1017ffff, 0x890153a1003ffff),
        ("SameParentOther", 0x0890153a1017ffff, 0x0890153a1013ffff),
        ("DifferentParent", 0x0890153a1017ffff, 0x0890153a10bbffff),
        ("DifferentParentFallback", 0x08908000001bffff, 0x08908000000fffff),
    ];

    for (case_name, origin, index) in cases.iter() {
        group.bench_with_input(
            BenchmarkId::new("h3on", case_name),
            &(index, origin),
            |b, &(index, origin)| bench_h3on(b, *index, *origin),
        );
        group.bench_with_input(
            BenchmarkId::new("h3o", case_name),
            &(index, origin),
            |b, &(index, origin)| bench_h3o(b, *index, *origin),
        );
    }

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
