use criterion::{black_box, BenchmarkId, Criterion};
use h3on::{CellIndex, Resolution};
use rayon::prelude::*;
use std::sync::Arc;
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



    // 다양한 크기의 데이터셋으로 테스트
    let dataset_sizes = [100, 1000, 10000, 100000];
    
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
    
    // 3. 대용량 데이터 처리: 모든 벤치마크 실행
    let large_dataset = generate_large_dataset(500000);
    println!("Large dataset: generated {} cells", large_dataset.len());
    
    // 대용량 데이터에서도 Sequential, Parallel, NUMA 모두 실행
    group.bench_function("h3on/Large_Dataset_Sequential", |b| {
        bench_h3on_sequential(b, &large_dataset)
    });
    
    group.bench_function("h3on/Large_Dataset_Parallel", |b| {
        bench_h3on_parallel(b, &large_dataset)
    });
    
    group.bench_function("h3on/Large_Dataset_NUMA", |b| {
        bench_h3on_numa_large(b, &large_dataset)
    });
    
    // h3o는 단일 스레드 기반이므로 병렬화 벤치마크 제거
    // group.bench_function("h3o/Large_Dataset_Parallel", |b| {
    //     bench_h3o_parallel_large(b, &large_dataset)
    // });
    
    // 4. Locality 테스트: 지역적으로 가까운 셀들
    let locality_dataset = generate_locality_dataset(10000);
    println!("Locality dataset: generated {} cells", locality_dataset.len());
    
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
    let base_cell = CellIndex::try_from(0x89283080ddbffff).expect("base cell");
    
    for i in 0..size {
        // 해상도별로 다른 셀 생성 (0-14)
        let res = (i % 15) as u8;
        let resolution = Resolution::try_from(res).unwrap();
        
        // 부모 셀에서 자식 셀 생성
        if let Some(cell) = base_cell.children(resolution).nth(i % 100) {
            cells.push(cell);
        }
        
        // 충분한 셀이 생성되지 않으면 다른 방법으로 추가
        if cells.len() < i + 1 {
            // 다른 해상도의 base cell 사용
            let alt_res = ((i + 7) % 15) as u8;
            let alt_resolution = Resolution::try_from(alt_res).unwrap();
            if let Some(cell) = base_cell.children(alt_resolution).nth((i + 13) % 50) {
                cells.push(cell);
            }
        }
    }
    
    // 최소한 size만큼의 셀을 보장
    while cells.len() < size {
        let extra_res = (cells.len() % 15) as u8;
        let extra_resolution = Resolution::try_from(extra_res).unwrap();
        if let Some(cell) = base_cell.children(extra_resolution).nth(cells.len() % 200) {
            cells.push(cell);
        } else {
            break; // 더 이상 생성할 수 없으면 중단
        }
    }
    
    println!("Generated {} cells for size {}", cells.len(), size);
    cells
}

fn generate_large_dataset(size: usize) -> Vec<CellIndex> {
    let mut cells = Vec::with_capacity(size);
    let base_cells = [
        0x89283080ddbffff,
        0x89283080c37ffff,
        0x89283080c27ffff,
        0x89283080d53ffff,
        0x89283080dcfffff,
        0x89283080dc3ffff,
    ];
    
    // 더 간단하고 확실한 방법: 각 base cell에서 직접 셀 생성
    for &base_val in &base_cells {
        if cells.len() >= size {
            break;
        }
        
        let base_cell = CellIndex::try_from(base_val).expect("base cell");
        
        // 해상도 0부터 시작하여 충분한 셀 생성
        for res in 0..15u8 {  // 0-14 해상도 모두 시도
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
    
    // 추가 셀 생성으로 부족분 보충
    if cells.len() < size {
        let mut extra_count = 0;
        while cells.len() < size && extra_count < size * 10 {
            let base_idx = extra_count % base_cells.len();
            let base_cell = CellIndex::try_from(base_cells[base_idx]).expect("base cell");
            let res = (extra_count % 15) as u8;
            let resolution = Resolution::try_from(res).unwrap();
            
            // 다른 인덱스 사용하여 중복 방지
            if let Some(cell) = base_cell.children(resolution).nth(extra_count % 1000) {
                cells.push(cell);
            }
            extra_count += 1;
        }
    }
    
    println!("Generated {} cells for large dataset size {} (target: {})", 
             cells.len(), size, size);
    cells
}

fn generate_locality_dataset(size: usize) -> Vec<CellIndex> {
    let mut cells = Vec::with_capacity(size);
    let center = CellIndex::try_from(0x89283080ddbffff).expect("center cell");
    
    // 중심 셀 주변의 지역적으로 가까운 셀들 생성
    let disk_cells: Vec<_> = center.grid_disk::<Vec<_>>(5).into_iter().collect();
    
    for i in 0..size {
        let cell_idx = i % disk_cells.len();
        if let Some(cell) = disk_cells.get(cell_idx) {
            cells.push(*cell);
        }
    }
    
    // 충분한 셀이 생성되지 않으면 다른 방법으로 추가
    while cells.len() < size {
        let extra_center = CellIndex::try_from(0x89283080c37ffff).expect("extra center cell");
        let extra_disk_cells: Vec<_> = extra_center.grid_disk::<Vec<_>>(3).into_iter().collect();
        
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
                    let more_cells: Vec<_> = center_cell.grid_disk::<Vec<_>>(2).into_iter().collect();
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
    
    println!("Generated {} cells for locality dataset size {}", cells.len(), size);
    cells
}

// -----------------------------------------------------------------------------

fn bench_h3on_sequential(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    use criterion::BatchSize;
    
    b.iter_batched(
        || data.to_vec(), // setup: 데이터 복사
        |data_copy| {
            let result: Vec<_> = data_copy.iter()
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
        BatchSize::LargeInput // 큰 입력에 최적화된 배치 크기
    );
}

fn bench_h3on_parallel(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    use criterion::BatchSize;
    
    b.iter_batched(
        || Arc::new(data.to_vec()), // setup: Arc로 감싼 데이터 준비
        |data_arc| {
            let result: Vec<_> = data_arc.par_iter()
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
        BatchSize::LargeInput
    );
}

fn bench_h3on_numa(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    use criterion::BatchSize;
    
    // NUMA 컨텍스트를 1회만 초기화 (벤치마크 루프 외부)
    #[cfg(feature = "numa")]
    let numa_ctx = {
        let ctx = init_numa_once(data.len());
        // 🚀 해당 벤치마크의 NUMA 설정 정보를 한 번만 출력 (메모리 할당 확인용)
        static PRINTED: AtomicBool = AtomicBool::new(false);
        if !PRINTED.fetch_or(true, Ordering::Relaxed) {
            println!("NUMA Setup for {} cells: buffer sizes: {:?}", data.len(), ctx.buffer_sizes);
        }
        ctx
    };
    
    b.iter_batched(
        || Arc::new(data.to_vec()), // setup: Arc로 감싼 데이터 준비
        |data_arc| {
            #[cfg(feature = "numa")]
            {
                // 이미 생성된 NUMA 컨텍스트 재사용
                let result = h3on::numa::build_numa_pool(&numa_ctx.topo, numa_ctx.buffer_sizes, || {
                    data_arc.par_iter()
                        .with_min_len(100)
                        .map(|&cell| {
                            let neighbors = cell.grid_disk::<Vec<_>>(2);
                            let area = cell.area_km2();
                            let boundary = cell.boundary();
                            (neighbors.len(), area, boundary.len())
                        })
                        .collect::<Vec<_>>()
                });
                black_box(result)
            }
            
            #[cfg(not(feature = "numa"))]
            {
                // 기본 병렬 처리
                let result: Vec<_> = data_arc.par_iter()
                    .map(|&cell| {
                        let neighbors = cell.grid_disk::<Vec<_>>(2);
                        let area = cell.area_km2();
                        let boundary = cell.boundary();
                        (neighbors.len(), area, boundary.len())
                    })
                    .collect();
                
                black_box(result)
            }
        },
        BatchSize::LargeInput
    );
}

fn bench_h3on_numa_large(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    use criterion::BatchSize;
    
    // NUMA 컨텍스트를 1회만 초기화 (벤치마크 루프 외부)
    #[cfg(feature = "numa")]
    let numa_ctx = {
        let ctx = init_numa_once(data.len() * 2); // 대용량 데이터용 버퍼 크기
        // 🚀 해당 벤치마크의 NUMA 설정 정보를 한 번만 출력 (메모리 할당 확인용)
        static PRINTED_LARGE: AtomicBool = AtomicBool::new(false);
        if !PRINTED_LARGE.fetch_or(true, Ordering::Relaxed) {
            println!("NUMA Large Setup for {} cells: buffer sizes: {:?}", data.len(), ctx.buffer_sizes);
        }
        ctx
    };
    
    b.iter_batched(
        || Arc::new(data.to_vec()), // setup: Arc로 감싼 데이터 준비
        |data_arc| {
            #[cfg(feature = "numa")]
            {
                // 이미 생성된 NUMA 컨텍스트 재사용
                let result = h3on::numa::build_numa_pool(&numa_ctx.topo, numa_ctx.buffer_sizes, || {
                    data_arc.par_iter()
                        .with_min_len(1000)
                        .map(|&cell| {
                            // 대용량 데이터에 최적화된 연산
                            let disk = cell.grid_disk::<Vec<_>>(3);
                            let distances = cell.grid_disk_distances::<Vec<_>>(3);
                            (disk.len(), distances.len())
                        })
                        .collect::<Vec<_>>()
                });
                black_box(result)
            }
            
            #[cfg(not(feature = "numa"))]
            {
                // 기본 병렬 처리
                let result: Vec<_> = data_arc.par_iter()
                    .with_min_len(1000)
                    .map(|&cell| {
                        let disk = cell.grid_disk::<Vec<_>>(3);
                        let distances = cell.grid_disk_distances::<Vec<_>>(3);
                        (disk.len(), distances.len())
                    })
                    .collect();
                
                black_box(result)
            }
        },
        BatchSize::LargeInput
    );
}

fn bench_h3on_locality(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    let data = Arc::new(data.to_vec());
    
    b.iter(|| {
        // 지역성 최적화: 가까운 셀들을 그룹화하여 처리
        let mut results = Vec::new();
        
        for chunk in data.chunks(100) {
            let chunk_results: Vec<_> = chunk.par_iter()
                .map(|&cell| {
                    // 지역적으로 가까운 셀들에 대한 연산
                    let neighbors = cell.grid_disk::<Vec<_>>(1);
                    let local_area = neighbors.iter()
                        .map(|n| n.area_km2())
                        .sum::<f64>();
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
    let h3o_data: Vec<h3o::CellIndex> = data.iter()
        .map(|&cell| h3o::CellIndex::try_from(u64::from(cell)).expect("h3o cell"))
        .collect();
    
    b.iter(|| {
        let result: Vec<_> = h3o_data.iter()
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
