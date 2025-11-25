use criterion::{BatchSize, BenchmarkId, Criterion};
use h3on::CellIndex;
use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
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

/// Reservoir Sampling을 사용한 메모리 효율적 랜덤 샘플링
fn reservoir_sample(
    iterator: &mut ZstCellIterator,
    sample_size: usize,
    rng: &mut ChaCha8Rng,
) -> Vec<CellIndex> {
    use rand::Rng;

    let mut reservoir = Vec::with_capacity(sample_size);
    let mut count = 0usize;

    // 처음 sample_size개는 무조건 추가
    for cell in iterator.by_ref().take(sample_size) {
        reservoir.push(cell);
        count += 1;
    }

    // 나머지는 확률적으로 교체
    for cell in iterator {
        count += 1;
        let j = rng.gen_range(0..count);
        if j < sample_size {
            reservoir[j] = cell;
        }
    }

    reservoir
}

fn load_cells_from_zst(size: usize, random: bool) -> Result<Vec<CellIndex>> {
    let project_root = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    let dataset_path = Path::new(&project_root)
        .join("dataset")
        .join("res9_cells.zst");

    println!("Loading cells from {:?}", dataset_path);

    let mut iterator = ZstCellIterator::from_file(&dataset_path)?;

    if random {
        // Reservoir Sampling을 사용한 메모리 효율적 랜덤 샘플링
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let cells = reservoir_sample(&mut iterator, size, &mut rng);

        println!(
            "Loaded {} random cells from dataset using reservoir sampling",
            cells.len()
        );
        Ok(cells)
    } else {
        // 순차 로드 (기존 방식)
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
}

fn generate_test_dataset(size: usize) -> Vec<CellIndex> {
    load_cells_from_zst(size, true).expect("Failed to load cells from zst file")
}

fn generate_locality_dataset(size: usize) -> Vec<CellIndex> {
    // locality dataset은 순차적으로 로드 (지역성 가정)
    load_cells_from_zst(size, false)
        .expect("Failed to load cells from zst file")
}

// -----------------------------------------------------------------------------

fn bench_h3on_sequential(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    b.iter(|| {
        let result = data
            .iter()
            .map(|&cell| {
                let neighbors = cell.grid_disk::<Vec<_>>(2);
                let area = cell.area_km2();
                let boundary = cell.boundary();
                (neighbors.len(), area, boundary.len())
            })
            .collect::<Vec<_>>();
        black_box(result);
    });
}

fn bench_h3on_parallel(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    b.iter(|| {
        let result = data
            .par_iter()
            .map(|&cell| {
                let neighbors = cell.grid_disk::<Vec<_>>(2);
                let area = cell.area_km2();
                let boundary = cell.boundary();
                (neighbors.len(), area, boundary.len())
            })
            .collect::<Vec<_>>();
        black_box(result);
    })
}

fn bench_h3on_numa(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    // NUMA 컨텍스트를 벤치마크 밖에서 한 번만 생성
    let numa_ctx = init_numa_once(data.len());
    println!(
        "NUMA Setup for {} cells: buffer sizes: {:?}",
        data.len(),
        numa_ctx.buffer_sizes
    );

    b.iter(|| {
        let result = h3on::numa::build_numa_pool(
            &numa_ctx.topo,
            numa_ctx.buffer_sizes,
            || {
                data.par_iter()
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
    });
}

fn bench_h3on_locality(b: &mut criterion::Bencher<'_>, data: &[CellIndex]) {
    b.iter(|| {
        // 지역성 최적화: 가까운 셀들을 그룹화하여 처리
        let results: Vec<_> = data
            .chunks(100)
            .flat_map(|chunk| {
                chunk
                    .par_iter()
                    .map(|&cell| {
                        // 지역적으로 가까운 셀들에 대한 연산
                        let neighbors = cell.grid_disk::<Vec<_>>(1);
                        let local_area =
                            neighbors.iter().map(|n| n.area_km2()).sum::<f64>();
                        (neighbors.len(), local_area)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        black_box(results)
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
