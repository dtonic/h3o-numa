use criterion::{BenchmarkId, Criterion};
use h3on::CellIndex;
use rayon::prelude::*;
use std::{
    fs::File,
    hint::black_box,
    io::{BufReader, Error, ErrorKind, Read, Result},
    path::Path,
    sync::Arc,
};
use zstd::stream::read::Decoder;

// -----------------------------------------------------------------------------
// NUMA 컨텍스트 구조체 (1회 초기화용)
struct NumaContext {
    topo: h3on::numa::NumaTopology,
    buffer_sizes: (usize, usize, usize),
}

fn init_numa_once(data_len: usize) -> NumaContext {
    let topo = h3on::numa::init_numa();
    let buffer_sizes = h3on::numa::estimate_buffer_sizes(15, data_len * 10);
    NumaContext { topo, buffer_sizes }
}

// -----------------------------------------------------------------------------

struct ZstCellIterator {
    decoder: Decoder<'static, BufReader<File>>,
    buf: [u8; 8],
}

impl ZstCellIterator {
    pub fn from_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut decoder = Decoder::new(file)?;
        // Set window log max to handle large compressed files (2GB window size)
        decoder.window_log_max(31)?;
        Ok(Self {
            decoder,
            buf: [0u8; 8],
        })
    }
}

impl Iterator for ZstCellIterator {
    type Item = CellIndex;

    fn next(&mut self) -> Option<Self::Item> {
        match self.decoder.read_exact(&mut self.buf) {
            Ok(_) => CellIndex::try_from(u64::from_le_bytes(self.buf)).ok(),
            Err(_) => None,
        }
    }
}

// -----------------------------------------------------------------------------

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("NUMA_Parallel_Performance");

    // 다양한 크기의 데이터셋으로 테스트 (linear 증가), 800000개 이상은 데이터 생성이 충분하지 않음
    // let dataset_sizes = [100];
    let dataset_sizes = [100, 1000, 10000, 100000, 200000, 400000, 600000];

    for &size in &dataset_sizes {
        let test_data = generate_test_dataset(size);

        // ✅ 데이터 생성 확인 로깅 추가
        println!("Dataset size {}: generated {} cells", size, test_data.len());

        // 1. 단일 스레드 vs 병렬 처리 비교
        group.bench_with_input(
            BenchmarkId::new("h3on/Sequential", size),
            &test_data,
            |b, data| bench_h3on_sequential(b, data),
        );

        group.bench_with_input(
            BenchmarkId::new("h3on/Parallel", size),
            &test_data,
            |b, data| bench_h3on_parallel(b, data),
        );

        // 2. h3o와의 비교
        group.bench_with_input(
            BenchmarkId::new("h3o/Sequential", size),
            &test_data,
            |b, data| bench_h3o_sequential(b, data),
        );

        // 3. 단순 복잡도 검증
        group.bench_with_input(
            BenchmarkId::new("Complexity_Check", size),
            &test_data,
            |b, data| bench_complexity_check(b, data),
        );

        // 4. NUMA 최적화 (마지막에 실행 - hwlocality 초기화가 무거움)
        group.bench_with_input(
            BenchmarkId::new("h3on/NUMA_Optimized", size),
            &test_data,
            |b, data| bench_h3on_numa(b, data),
        );

        // h3o는 단일 스레드 기반이므로 병렬화 벤치마크 제거
        // group.bench_with_input(
        //     BenchmarkId::new("h3o/Parallel", size),
        //     &test_data,
        //     |b, data| bench_h3o_parallel(b, data),
        // );
    }

    // 3. 대용량 데이터 처리: 500,000 cells (기존 호환성 유지)
    let large_dataset = generate_test_dataset(500000); // generate_large_dataset 대신 generate_test_dataset 사용
    println!("Large dataset: generated {} cells", large_dataset.len());

    // 대용량 데이터에서도 Sequential, Parallel, NUMA 모두 실행
    group.bench_function("h3on/Large_Dataset_Sequential", |b| {
        bench_h3on_sequential(b, &large_dataset)
    });

    group.bench_function("h3on/Large_Dataset_Parallel", |b| {
        bench_h3on_parallel(b, &large_dataset)
    });

    group.bench_function("h3on/Large_Dataset_NUMA", |b| {
        bench_h3on_numa(b, &large_dataset) // bench_h3on_numa_large 대신 bench_h3on_numa 사용
    });

    // 4. Locality 테스트: 지역적으로 가까운 셀들
    let locality_dataset = generate_locality_dataset(10000);
    println!(
        "Locality dataset: generated {} cells",
        locality_dataset.len()
    );

    group.bench_function("h3on/Locality_Optimized", |b| {
        bench_h3on_locality(b, &locality_dataset)
    });

    // h3o는 단일 스레드 기반이므로 병렬화 벤치마크 제거
    // group.bench_function("h3o/Locality_Optimized", |b| {
    //     bench_h3o_locality(b, &locality_dataset)
    // });

    group.finish();
}

// -----------------------------------------------------------------------------

fn load_cells_from_zst(size: usize) -> Result<Vec<CellIndex>> {
    let project_root = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    let dataset_path = Path::new(&project_root)
        .join("dataset")
        .join("res9_cells.zst");

    println!("Loading cells from {:?}", dataset_path);

    let mut iterator = ZstCellIterator::from_file(&dataset_path)?;
    let cells: Vec<CellIndex> = iterator.by_ref().take(size).collect();

    if cells.len() < size {
        println!(
            "Warning: Only {} cells available in dataset (requested {})",
            cells.len(),
            size
        );
    } else {
        println!("Loaded {} cells from dataset", cells.len());
    }

    Ok(cells)
}

fn generate_test_dataset(size: usize) -> Vec<CellIndex> {
    load_cells_from_zst(size).expect("Failed to load cells from zst file")
}

fn generate_locality_dataset(size: usize) -> Vec<CellIndex> {
    // locality dataset도 zst 파일에서 로드하되, 앞부분의 연속된 셀들을 사용 (지역성 가정)
    load_cells_from_zst(size).expect("Failed to load cells from zst file")
}

// -----------------------------------------------------------------------------

fn bench_h3on_sequential(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    use criterion::BatchSize;

    b.iter_batched(
        || data.to_vec(), // setup: 데이터 복사
        |data_copy| {
            let result: Vec<_> = data_copy
                .iter()
                .map(|&cell| {
                    // 각 셀에 대해 복잡한 연산 수행
                    let neighbors = cell.grid_disk::<Vec<_>>(2);
                    let area = cell.area_km2();
                    let boundary = cell.boundary();
                    (neighbors.len(), area, boundary.len())
                })
                .collect();

            // black_box로 결과를 실제로 사용하여 dead code elimination 방지
            black_box(result)
        },
        BatchSize::LargeInput, // 큰 입력에 최적화된 배치 크기
    );
}

fn bench_h3on_parallel(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    use criterion::BatchSize;

    b.iter_batched(
        || Arc::new(data.to_vec()), // setup: Arc로 감싼 데이터 준비
        |data_arc| {
            let result: Vec<_> = data_arc
                .par_iter()
                .map(|&cell| {
                    // 병렬로 복잡한 연산 수행
                    let neighbors = cell.grid_disk::<Vec<_>>(2);
                    let area = cell.area_km2();
                    let boundary = cell.boundary();
                    (neighbors.len(), area, boundary.len())
                })
                .collect();

            // black_box로 결과를 실제로 사용
            black_box(result)
        },
        BatchSize::LargeInput,
    );
}

fn bench_h3on_numa(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    use criterion::BatchSize;

    b.iter_batched(
        || {
            let numa_ctx = {
                let ctx = init_numa_once(data.len());
                // 🚀 해당 벤치마크의 NUMA 설정 정보를 한 번만 출력 (메모리 할당 확인용)
                use std::sync::atomic::{AtomicBool, Ordering};
                static PRINTED: AtomicBool = AtomicBool::new(false);
                if !PRINTED.fetch_or(true, Ordering::Relaxed) {
                    println!(
                        "NUMA Setup for {} cells: buffer sizes: {:?}",
                        data.len(),
                        ctx.buffer_sizes
                    );
                }
                ctx
            };

            (Arc::new(data.to_vec()), numa_ctx)
        },
        |(data_arc, numa_ctx)| {
            // 이미 생성된 NUMA 컨텍스트 재사용
            let result = h3on::numa::build_numa_pool(
                &numa_ctx.topo,
                numa_ctx.buffer_sizes,
                || {
                    data_arc
                        .par_iter()
                        .with_min_len(100)
                        .map(|&cell| {
                            let neighbors = cell.grid_disk::<Vec<_>>(2);
                            let area = cell.area_km2();
                            let boundary = cell.boundary();
                            (neighbors.len(), area, boundary.len())
                        })
                        .collect::<Vec<_>>()
                },
            );
            black_box(result)
        },
        BatchSize::LargeInput,
    );
}

fn bench_h3on_locality(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    let data = Arc::new(data.to_vec());

    b.iter(|| {
        // 지역성 최적화: 가까운 셀들을 그룹화하여 처리
        let mut results = Vec::new();

        for chunk in data.chunks(100) {
            let chunk_results: Vec<_> = chunk
                .par_iter()
                .map(|&cell| {
                    // 지역적으로 가까운 셀들에 대한 연산
                    let neighbors = cell.grid_disk::<Vec<_>>(1);
                    let local_area =
                        neighbors.iter().map(|n| n.area_km2()).sum::<f64>();
                    (neighbors.len(), local_area)
                })
                .collect();
            results.extend(chunk_results);
        }

        results
    });
}

// -----------------------------------------------------------------------------

fn bench_h3o_sequential(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    let h3o_data: Vec<h3on::CellIndex> = data
        .iter()
        .map(|&cell| {
            h3on::CellIndex::try_from(u64::from(cell)).expect("h3o cell")
        })
        .collect();

    b.iter(|| {
        let result: Vec<_> = h3o_data
            .iter()
            .map(|&cell| {
                let neighbors = cell.grid_disk::<Vec<_>>(2);
                let area = cell.area_km2();
                let boundary = cell.boundary();
                (neighbors.len(), area, boundary.len())
            })
            .collect();

        // black_box로 결과를 실제로 사용
        black_box(result)
    });
}

// h3o는 단일 스레드 기반이므로 병렬화 함수들 제거
// fn bench_h3o_parallel(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) { ... }
// fn bench_h3o_parallel_large(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) { ... }
// fn bench_h3o_locality(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) { ... }

// -----------------------------------------------------------------------------

/// 복잡도 검증을 위한 단순 벤치마크
fn bench_complexity_check(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    b.iter(|| {
        // 단순한 O(n) 연산: 각 셀에 대해 1씩 더하기
        let sum: u64 = data.iter().map(|_| 1).sum();

        // black_box로 결과를 실제로 사용
        black_box(sum)
    });
}
