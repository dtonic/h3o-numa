use criterion::{BenchmarkId, Criterion};
use h3on::{CellIndex, Resolution};
use rayon::prelude::*;
use std::sync::Arc;

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("NUMA_Parallel_Performance");

    // 다양한 크기의 데이터셋으로 테스트
    let dataset_sizes = [100, 1000, 10000, 100000];
    
    for &size in &dataset_sizes {
        let test_data = generate_test_dataset(size);
        
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
        
        // h3o는 단일 스레드 기반이므로 병렬화 벤치마크 제거
        // group.bench_with_input(
        //     BenchmarkId::new("h3o/Parallel", size),
        //     &test_data,
        //     |b, data| bench_h3o_parallel(b, data),
        // );
    }
    
    // 3. NUMA 특화 테스트: 대용량 데이터 처리
    let large_dataset = generate_large_dataset(500000);
    group.bench_function("h3on/Large_Dataset_NUMA", |b| {
        bench_h3on_numa_large(b, &large_dataset)
    });
    
    // h3o는 단일 스레드 기반이므로 병렬화 벤치마크 제거
    // group.bench_function("h3o/Large_Dataset_Parallel", |b| {
    //     bench_h3o_parallel_large(b, &large_dataset)
    // });
    
    // 4. Locality 테스트: 지역적으로 가까운 셀들
    let locality_dataset = generate_locality_dataset(10000);
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
        // 다양한 해상도와 위치의 셀 생성
        let res = (i % 15) as u8; // 0-14 해상도
        let offset = i as u64;
        
        // u64로 변환 후 오프셋 추가
        let cell_value = u64::from(base_cell) + offset;
        if let Ok(cell) = CellIndex::try_from(cell_value) {
            if cell.resolution() == Resolution::try_from(res).unwrap() {
                cells.push(cell);
            }
        }
    }
    
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
    
    for i in 0..size {
        let base_idx = i % base_cells.len();
        let base_cell = CellIndex::try_from(base_cells[base_idx]).expect("base cell");
        let res = (i % 12) as u8; // 0-11 해상도
        
        if let Some(cell) = base_cell.children(Resolution::try_from(res).unwrap()).next() {
            cells.push(cell);
        }
    }
    
    cells
}

fn generate_locality_dataset(size: usize) -> Vec<CellIndex> {
    let mut cells = Vec::with_capacity(size);
    let center = CellIndex::try_from(0x89283080ddbffff).expect("center cell");
    
    // 중심 셀 주변의 지역적으로 가까운 셀들 생성
    for i in 0..size {
        if let Some(neighbor) = center.grid_disk::<Vec<_>>(3).get(i % 37) {
            cells.push(*neighbor);
        }
    }
    
    cells
}

// -----------------------------------------------------------------------------

fn bench_h3on_sequential(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    b.iter(|| {
        data.iter()
            .map(|&cell| {
                // 각 셀에 대해 복잡한 연산 수행
                let neighbors = cell.grid_disk::<Vec<_>>(2);
                let area = cell.area_km2();
                let boundary = cell.boundary();
                (neighbors.len(), area, boundary.len())
            })
            .collect::<Vec<_>>()
    });
}

fn bench_h3on_parallel(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    let data = Arc::new(data.to_vec());
    
    b.iter(|| {
        data.par_iter()
            .map(|&cell| {
                // 병렬로 복잡한 연산 수행
                let neighbors = cell.grid_disk::<Vec<_>>(2);
                let area = cell.area_km2();
                let boundary = cell.boundary();
                (neighbors.len(), area, boundary.len())
            })
            .collect::<Vec<_>>()
    });
}

fn bench_h3on_numa(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    let data = Arc::new(data.to_vec());
    
    b.iter(|| {
        #[cfg(feature = "numa")]
        {
            // 실제 NUMA 최적화: NUMA 노드별로 데이터 분할
            use crate::numa::{init_numa, build_numa_pool, estimate_buffer_sizes};
            
            let topo = init_numa();
            let buffer_sizes = estimate_buffer_sizes(15, data.len() * 10);
            
            build_numa_pool(&topo, buffer_sizes, || {
                data.par_iter()
                    .with_min_len(100)
                    .map(|&cell| {
                        let neighbors = cell.grid_disk::<Vec<_>>(2);
                        let area = cell.area_km2();
                        let boundary = cell.boundary();
                        (neighbors.len(), area, boundary.len())
                    })
                    .collect::<Vec<_>>()
            })
        }
        
        #[cfg(not(feature = "numa"))]
        {
            // 기본 병렬 처리
            data.par_iter()
                .map(|&cell| {
                    let neighbors = cell.grid_disk::<Vec<_>>(2);
                    let area = cell.area_km2();
                    let boundary = cell.boundary();
                    (neighbors.len(), area, boundary.len())
                })
                .collect::<Vec<_>>()
        }
    });
}

fn bench_h3on_numa_large(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    let data = Arc::new(data.to_vec());
    
    b.iter(|| {
        #[cfg(feature = "numa")]
        {
            // 실제 NUMA 최적화: NUMA 노드별로 데이터 분할
            use crate::numa::{init_numa, build_numa_pool, estimate_buffer_sizes};
            
            let topo = init_numa();
            let buffer_sizes = estimate_buffer_sizes(15, data.len() * 20);
            
            build_numa_pool(&topo, buffer_sizes, || {
                data.par_iter()
                    .with_min_len(1000)
                    .map(|&cell| {
                        // 대용량 데이터에 최적화된 연산
                        let disk = cell.grid_disk::<Vec<_>>(3);
                        let distances = cell.grid_disk_distances::<Vec<_>>(3);
                        (disk.len(), distances.len())
                    })
                    .collect::<Vec<_>>()
            })
        }
        
        #[cfg(not(feature = "numa"))]
        {
            // 기본 병렬 처리
            data.par_iter()
                .with_min_len(1000)
                .map(|&cell| {
                    let disk = cell.grid_disk::<Vec<_>>(3);
                    let distances = cell.grid_disk_distances::<Vec<_>>(3);
                    (disk.len(), distances.len())
                })
                .collect::<Vec<_>>()
        }
    });
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
        h3o_data.iter()
            .map(|&cell| {
                let neighbors = cell.grid_disk::<Vec<_>>(2);
                let area = cell.area_km2();
                let boundary = cell.boundary();
                (neighbors.len(), area, boundary.len())
            })
            .collect::<Vec<_>>()
    });
}

// h3o는 단일 스레드 기반이므로 병렬화 함수들 제거
// fn bench_h3o_parallel(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) { ... }
// fn bench_h3o_parallel_large(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) { ... }
// fn bench_h3o_locality(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) { ... }
