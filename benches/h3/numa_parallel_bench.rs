use criterion::{BenchmarkId, Criterion};
use h3on::{CellIndex, Resolution};
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::hint::black_box;
#[cfg(any(feature = "rayon", feature = "numa"))]
use std::sync::Arc;
#[cfg(feature = "numa")]
use std::sync::atomic::{AtomicBool, Ordering};

// -----------------------------------------------------------------------------
// NUMA 컨텍스트 구조체 (1회 초기화용)

#[cfg(feature = "numa")]
struct NumaContext {
    topo: h3on::numa::NumaTopology,
    buffer_sizes: (usize, usize, usize),
}

#[cfg(feature = "numa")]
fn init_numa_once(data_len: usize) -> NumaContext {
    let topo = h3on::numa::init_numa();
    let buffer_sizes = h3on::numa::estimate_buffer_sizes(15, data_len * 10);
    NumaContext { topo, buffer_sizes }
}

// -----------------------------------------------------------------------------

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("NUMA_Parallel_Performance");

    // 다양한 크기의 데이터셋으로 테스트 (linear 증가), 800000개 이상은 데이터 생성이 충분하지 않음
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

        #[cfg(feature = "rayon")]
        group.bench_with_input(
            BenchmarkId::new("h3on/Parallel", size),
            &test_data,
            |b, data| bench_h3on_parallel(b, data),
        );

        #[cfg(feature = "numa")]
        group.bench_with_input(
            BenchmarkId::new("h3on/NUMA_Optimized", size),
            &test_data,
            |b, data| bench_h3on_numa(b, data),
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

    #[cfg(feature = "rayon")]
    group.bench_function("h3on/Large_Dataset_Parallel", |b| {
        bench_h3on_parallel(b, &large_dataset)
    });

    #[cfg(feature = "numa")]
    group.bench_function("h3on/Large_Dataset_NUMA", |b| {
        bench_h3on_numa(b, &large_dataset) // bench_h3on_numa_large 대신 bench_h3on_numa 사용
    });

    // 4. Locality 테스트: 지역적으로 가까운 셀들
    let locality_dataset = generate_locality_dataset(10000);
    println!(
        "Locality dataset: generated {} cells",
        locality_dataset.len()
    );

    #[cfg(feature = "rayon")]
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

fn generate_test_dataset(size: usize) -> Vec<CellIndex> {
    let mut cells = Vec::with_capacity(size);

    // 여러 base cell을 사용하여 더 많은 고유한 셀 생성 (유효한 base cell만 사용)
    let base_cells = [
        0x89283080ddbffff, // 유효한 base cell
        0x89283080c37ffff, // 유효한 base cell
        0x89283080c27ffff, // 유효한 base cell
        0x89283080d53ffff, // 유효한 base cell
        0x89283080dcfffff, // 유효한 base cell
        0x89283080dc3ffff, // 유효한 base cell
    ];

    // 더 안전한 방법: 각 base cell에서 해상도별로 체계적으로 셀 생성
    for &base_val in &base_cells {
        if cells.len() >= size {
            break;
        }

        let base_cell = CellIndex::try_from(base_val).expect("valid base cell");

        // 해상도 0부터 14까지 순차적으로 생성
        for res in 0..15u8 {
            if cells.len() >= size {
                break;
            }

            let resolution = Resolution::try_from(res).unwrap();

            // 각 해상도에서 사용 가능한 자식 셀들을 순차적으로 추가
            for child in base_cell.children(resolution) {
                if cells.len() >= size {
                    break;
                }
                cells.push(child);
            }
        }
    }

    // 부족한 경우 다른 base cell에서 추가 생성
    if cells.len() < size {
        let mut extra_base_idx = 0;
        while cells.len() < size && extra_base_idx < base_cells.len() * 5 {
            let base_cell = CellIndex::try_from(
                base_cells[extra_base_idx % base_cells.len()],
            )
            .expect("valid base cell");

            // 해상도 0-14를 순환하면서 추가 셀 생성
            for res in 0..15u8 {
                if cells.len() >= size {
                    break;
                }

                let resolution = Resolution::try_from(res).unwrap();

                // 다른 인덱스로 중복 방지
                let start_idx = (extra_base_idx + res as usize) % 100;
                for (i, child) in base_cell.children(resolution).enumerate() {
                    if i < start_idx {
                        continue;
                    }
                    if cells.len() >= size {
                        break;
                    }
                    cells.push(child);
                }
            }
            extra_base_idx += 1;
        }
    }

    println!(
        "Generated {} cells for size {} (target: {})",
        cells.len(),
        size,
        size
    );
    cells
}

fn generate_locality_dataset(size: usize) -> Vec<CellIndex> {
    let mut cells = Vec::with_capacity(size);
    let center = CellIndex::try_from(0x89283080ddbffff).expect("center cell");

    // 중심 셀 주변의 지역적으로 가까운 셀들 생성
    let disk_cells: Vec<_> =
        center.grid_disk::<Vec<_>>(5).into_iter().collect();

    for i in 0..size {
        let cell_idx = i % disk_cells.len();
        if let Some(cell) = disk_cells.get(cell_idx) {
            cells.push(*cell);
        }
    }

    // 충분한 셀이 생성되지 않으면 다른 방법으로 추가
    while cells.len() < size {
        let extra_center =
            CellIndex::try_from(0x89283080c37ffff).expect("extra center cell");
        let extra_disk_cells: Vec<_> =
            extra_center.grid_disk::<Vec<_>>(3).into_iter().collect();

        for (_i, cell) in extra_disk_cells.iter().enumerate() {
            if cells.len() >= size {
                break;
            }
            cells.push(*cell);
        }

        if cells.len() < size {
            // 더 많은 base cell에서 생성
            let more_centers = [
                0x89283080c27ffff,
                0x89283080d53ffff,
                0x89283080dcfffff,
                0x89283080dc3ffff,
            ];

            for center_val in &more_centers {
                if cells.len() >= size {
                    break;
                }
                if let Ok(center_cell) = CellIndex::try_from(*center_val) {
                    let more_cells: Vec<_> = center_cell
                        .grid_disk::<Vec<_>>(2)
                        .into_iter()
                        .collect();
                    for cell in more_cells {
                        if cells.len() >= size {
                            break;
                        }
                        cells.push(cell);
                    }
                }
            }
        }

        // 무한 루프 방지
        if cells.len() == 0 {
            break;
        }
    }

    println!(
        "Generated {} cells for locality dataset size {}",
        cells.len(),
        size
    );
    cells
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

#[cfg(feature = "rayon")]
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

#[cfg(feature = "numa")]
fn bench_h3on_numa(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    use criterion::BatchSize;

    // NUMA 컨텍스트를 1회만 초기화 (벤치마크 루프 외부)
    let numa_ctx = {
        let ctx = init_numa_once(data.len());
        // 🚀 해당 벤치마크의 NUMA 설정 정보를 한 번만 출력 (메모리 할당 확인용)
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

    b.iter_batched(
        || Arc::new(data.to_vec()), // setup: Arc로 감싼 데이터 준비
        |data_arc| {
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

#[cfg(feature = "rayon")]
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
    let h3o_data: Vec<h3o::CellIndex> = data
        .iter()
        .map(|&cell| {
            h3o::CellIndex::try_from(u64::from(cell)).expect("h3o cell")
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
