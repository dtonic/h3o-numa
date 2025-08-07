use criterion::{black_box, BatchSize, Bencher, Criterion};
use h3on::{CellIndex, Direction, Resolution};

pub fn bench(c: &mut Criterion) {
    const RESOLUTION: Resolution = Resolution::Three;

    let mut group = c.benchmark_group("compactCells");

    let cells = CellIndex::base_cells()
        .flat_map(|index| index.children(RESOLUTION))
        .collect::<Vec<_>>();
    group.bench_function("h3on/FullCompaction", |b| bench_h3on(b, &cells));
    group.bench_function("h3o/FullCompaction", |b| bench_h3o(b, &cells));
    group.bench_function("h3/FullCompaction", |b| bench_h3(b, &cells));

    let sparse = cells
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(idx, cell)| (idx % 33 != 0).then_some(cell))
        .collect::<Vec<_>>();
    group.bench_function("h3on/PartialCompaction", |b| bench_h3on(b, &sparse));
    group.bench_function("h3o/PartialCompaction", |b| bench_h3o(b, &sparse));
    group.bench_function("h3/PartialCompaction", |b| bench_h3(b, &sparse));

    let uncompactable = cells
        .iter()
        .copied()
        .filter(|cell| cell.direction_at(RESOLUTION) != Some(Direction::IK))
        .collect::<Vec<_>>();
    group.bench_function("h3on/NoCompaction", |b| bench_h3on(b, &uncompactable));
    group.bench_function("h3o/NoCompaction", |b| bench_h3o(b, &uncompactable));
    group.bench_function("h3/NoCompaction", |b| bench_h3(b, &uncompactable));

    group.finish();
}

// -----------------------------------------------------------------------------

fn bench_h3on(b: &mut Bencher<'_>, indexes: &[CellIndex]) {
    b.iter_batched(
        || indexes.to_owned(),
        |mut indexes| {
            CellIndex::compact(black_box(&mut indexes)).expect("compacted set")
        },
        BatchSize::SmallInput,
    )
}

fn bench_h3o(b: &mut Bencher<'_>, indexes: &[CellIndex]) {
    b.iter_batched(
        || {
            indexes
                .iter()
                .copied()
                .map(|cell| h3o::CellIndex::try_from(u64::from(cell)).expect("cell index"))
                .collect::<Vec<_>>()
        },
        |mut indexes| {
            h3o::CellIndex::compact(black_box(&mut indexes)).expect("compacted set")
        },
        BatchSize::SmallInput,
    )
}

fn bench_h3(b: &mut Bencher<'_>, indexes: &[CellIndex]) {
    let indexes = indexes.iter().copied().map(u64::from).collect::<Vec<_>>();
    let mut out = vec![0; indexes.len()];
    b.iter(|| unsafe {
        h3ron_h3_sys::compactCells(
            black_box(indexes.as_ptr()),
            out.as_mut_ptr(),
            indexes.len() as i64,
        )
    })
}
