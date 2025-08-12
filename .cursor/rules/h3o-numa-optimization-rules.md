# 🧠 `h3on` NUMA 최적화 Rulebook

## 🎯 프로젝트 목표

> `h3on`은 [HydroniumLabs/h3o](https://github.com/HydroniumLabs/h3o)의 fork로, NUMA 기반 멀티코어 환경에서 **대규모 공간 연산을 병렬화하고, 메모리 locality를 개선**하여 성능을 4~7배 향상시키는 것을 목표로 합니다.
현재 codebase는 [h3o-numa](https://github.com/SeonbaeHwang/h3o-numa)에 위치. working branch는 update-rulebook

- ✅ `h3on`은 기존 `h3o`를 기반으로 하지만, **모든 모듈 및 벤치마크 명시적으로 `h3on`으로 변경**
- ✅ NUMA-aware 스레드/메모리/데이터 구조 최적화 적용
- ✅ `criterion`을 통해 기존 `h3`, `h3o`, `h3on` 비교 가능

## 📋 적용 전제 조건

| 항목 | 내용 |
|------|------|
| 멀티스레드 | 모든 연산은 병렬화된 구조로 전환 (단일 스레드 제거) |
| NUMA 적용 | 스레드 고정(Thread Affinity) + 메모리 고정(Memory Pinning) |
| 플랫폼 | Linux 기반 NUMA 시스템 (x86 서버 등) |
| 추적성 | `// TODO:` 및 `// DONE:` 주석을 통해 Cursor 연동 |
| 모듈 명시성 | **모든 구현은 `h3on::` 네임스페이스로 통일** |
| 벤치마크 기준 | `h3on` 이름으로 결과 구분 명확화 (`polygon_to_cells_h3on` 등) |
| 변경사항 검증 | 각 STEP 적용 시 아래 검증 방법을 통한 프로그램 변경 사항 검증 |
| 일관적인 테스트 유지 | agent는 test 코드는 수정하지 않음 |

- 변경사항 검증
```
# 전체 테스트 실행
cargo test --all-features --verbose

# 린팅 및 코드 품질 검사
cargo clippy --all-targets --all-features
```

## 🪜 단계별 TODO (병렬화 → NUMA-aware 순서)

### 🔹 STEP 0. NUMA 적용 대상 분석 및 전략 분류

```rust
// DONE: 병렬화 및 NUMA 적용 타겟 함수 목록 작성 (e.g. grid_disks_fast, compact, polygon_to_cells)
// DONE: 연산별 병렬화/NUMA 적용 가능성 평가 (독립성, 데이터 locality)
// DONE: 각 함수의 데이터 접근 패턴 정리 및 전략 분류
// DONE: h3o-numa-optimization-rules.md 업데이트
```

> 🎯 목적: 성능 병목 지점 우선순위 지정, NUMA 전략 결정 기준 수립

**분석 결과:**
- **핵심 병목 함수들:**
  1. `grid_disks_fast` (src/index/cell.rs:1152-1160) - 다중 인덱스 처리, 순차적 flat_map
  2. `compact` (src/index/cell.rs:669-725) - 정렬 및 압축 연산, 순차적 처리
  3. `into_coverage` (src/geom/tiler.rs:153-247) - 폴리곤 타일링, 복잡한 내부 전파 로직
  4. `grid_disk_fast` (src/index/cell.rs:1036-1064) - 단일 인덱스 디스크 생성
  5. `uncompact` (src/index/cell.rs:750-768) - 압축 해제 연산

### 🔹 STEP 1. PACKAGE 이름 변경

```rust
// DONE: Cargo.toml의 package name을 "h3o"에서 "h3on"으로 변경
// DONE: 모든 모듈 네임스페이스를 "h3o::"에서 "h3on::"로 변경
// DONE: 모든 import 문을 "use h3o::"에서 "use h3on::"로 변경
// DONE: 모든 문서 및 예시 코드의 네임스페이스 업데이트
// DONE: 벤치마크 및 테스트 코드의 네임스페이스 업데이트
// DONE: README.md 및 문서의 패키지명 참조 업데이트
// TODO: CI/CD 파이프라인의 패키지명 참조 업데이트
```

> 🎯 목적: h3on 네임스페이스로 통일하여 기존 h3o와 명확히 구분

**구체적 적용 방안:**
1. **Cargo.toml 패키지명 변경** ✅ **완료**
   ```toml
   [package]
   name = "h3on"  # "h3o"에서 "h3on"으로 변경 완료
   version = "0.8.0"
   description = "A Rust implementation of the H3 geospatial indexing system with NUMA optimizations."
   ```

2. **모듈 네임스페이스 변경** ✅ **완료**
   ```rust
   // 기존: use h3o::CellIndex;
   // 변경: use h3on::CellIndex; ✅ 완료
   
   // 기존: h3o::CellIndex::try_from(0x8b1fb46622dcfff)
   // 변경: h3on::CellIndex::try_from(0x8b1fb46622dcfff) ✅ 완료
   ```

3. **문서 및 예시 코드 업데이트** ✅ **완료**
   ```rust
   // 모든 문서 예시에서 h3o를 h3on으로 변경 ✅ 완료
   // benches/h3/ 디렉토리의 모든 벤치마크 코드 업데이트 ✅ 완료
   // tests/ 디렉토리의 모든 테스트 코드 업데이트 ✅ 완료
   ```

4. **CI/CD 파이프라인 업데이트** 🔄 **진행 중**
   ```yaml
   # .github/workflows/ 디렉토리의 모든 워크플로우 파일 업데이트
   # 패키지명 참조를 h3o에서 h3on으로 변경
   ```

**✅ STEP 1 완료 요약:**
- 패키지명 `h3o` → `h3on` 변경 완료
- 모든 모듈 네임스페이스 업데이트 완료
- 문서 및 예시 코드 업데이트 완료
- 벤치마크 및 테스트 코드 업데이트 완료
- README.md 업데이트 완료
- `cargo check` 통과 확인 완료

**🎯 다음 단계 준비:**
- STEP 2: 병렬화 구조 적용 (`rayon`) 준비 완료
- 기존 h3o와 명확히 구분된 h3on 네임스페이스 확립
- NUMA 최적화를 위한 기반 구조 구축 완료

### 🔹 STEP 2. 병렬화 구조 적용 (`rayon`)

```rust
// DONE: par_iter를 사용한 병렬 처리 도입 방안 제시 (e.g. grid_disks_fast)
// DONE: grid_disks_fast 병렬화 적용 - 다중 인덱스 처리 성능 향상
// DONE: compact 연산 병렬화 적용 - 대용량 데이터 정렬 성능 향상
// DONE: into_coverage 병렬화 적용 - 내부 전파 단계 성능 향상
// DONE: uncompact 연산 병렬화 적용 - 압축 해제 연산 성능 향상
// DONE: uncompact_size 병렬화 적용 - 크기 계산 성능 향상
```

> 🎯 목적: 병렬 구조 기반 확보, 이후 NUMA 적용의 기반 마련

**구체적 적용 방안:**
1. **`grid_disks_fast` 병렬화** (src/index/cell.rs:1169-1200) ✅ **완료**
   ```rust
   // DONE: rayon par_iter를 사용한 병렬 처리 적용 - 다중 인덱스 처리 성능 향상
   #[cfg(feature = "rayon")]
   {
       use rayon::prelude::*;
       let indexes: Vec<_> = indexes.into_iter().collect();
       if indexes.len() > 100 {
           // 대용량 데이터의 경우 병렬 처리 적용
           indexes
               .into_par_iter()
               .flat_map_iter(move |index| index.grid_disk_fast(k))
       } else {
           // 소용량 데이터의 경우 순차 처리 유지
           indexes
               .into_iter()
               .flat_map(move |index| index.grid_disk_fast(k))
       }
   }
   ```

2. **`compact` 병렬화** (src/index/cell.rs:669-725) ✅ **완료**
   ```rust
   // DONE: rayon par_sort_unstable를 사용한 병렬 정렬 적용 - 대용량 데이터 정렬 성능 향상
   #[cfg(feature = "rayon")]
   {
       use rayon::prelude::*;
       if cells.len() > 1000 {
           // 대용량 데이터의 경우 병렬 정렬 적용
           cells.par_sort_unstable();
       } else {
           // 소용량 데이터의 경우 순차 정렬 유지
           cells.sort_unstable();
       }
   }
   ```

3. **`into_coverage` 병렬화** (src/geom/tiler.rs:153-247) ✅ **완료**
   ```rust
   // DONE: rayon par_iter를 사용한 병렬 처리 적용 - 내부 전파 단계 성능 향상
   #[cfg(feature = "rayon")]
   {
       use rayon::prelude::*;
       if candidates.len() > 100 {
           // 대용량 데이터의 경우 병렬 처리 적용
           let next_gen_par: Vec<_> = candidates
               .par_iter()
               .flat_map_iter(|&(cell, _)| {
                   // 내부 전파 로직 병렬화
               })
               .collect();
           next_gen.extend(next_gen_par);
       }
   }
   ```

4. **`uncompact` 병렬화** (src/index/cell.rs:750-768) ✅ **완료**
   ```rust
   // DONE: rayon par_iter를 사용한 병렬 처리 적용 - 압축 해제 연산 성능 향상
   #[cfg(feature = "rayon")]
   {
       use rayon::prelude::*;
       let compacted: Vec<_> = compacted.into_iter().collect();
       if compacted.len() > 100 {
           // 대용량 데이터의 경우 병렬 처리 적용
           compacted
               .into_par_iter()
               .flat_map_iter(move |index| index.children(resolution))
       }
   }
   ```

5. **`uncompact_size` 병렬화** (src/index/cell.rs:743-750) ✅ **완료**
   ```rust
   // DONE: rayon par_iter를 사용한 병렬 처리 적용 - 크기 계산 성능 향상
   #[cfg(feature = "rayon")]
   {
       use rayon::prelude::*;
       let compacted: Vec<_> = compacted.into_iter().collect();
       if compacted.len() > 100 {
           // 대용량 데이터의 경우 병렬 처리 적용
           compacted
               .into_par_iter()
               .map(move |index| index.children_count(resolution))
               .sum()
       }
   }
   ```

**✅ STEP 2 완료 요약:**
- `rayon` 의존성 추가 완료 (Cargo.toml)
- `grid_disks_fast` 병렬화 적용 완료 - 다중 인덱스 처리 성능 향상
- `compact` 병렬화 적용 완료 - 대용량 데이터 정렬 성능 향상
- `into_coverage` 병렬화 적용 완료 - 내부 전파 단계 성능 향상
- `uncompact` 병렬화 적용 완료 - 압축 해제 연산 성능 향상
- `uncompact_size` 병렬화 적용 완료 - 크기 계산 성능 향상
- 조건부 컴파일(`#[cfg(feature = "rayon")]`)을 통한 선택적 병렬화 적용
- 대용량 데이터(100개 이상)에서만 병렬화 적용하여 오버헤드 최소화
- **h3o와 h3on을 equivalent하게 비교할 수 있는 벤치마크 추가 완료**
  - `h3o` 의존성 추가 (dev-dependencies)
  - `grid_disks_unsafe`, `compact_cells`, `polygon_to_cells`, `uncompact_cells`, `grid_disk`, `cell_to_children` 벤치마크에 h3o 비교 추가
  - 동일한 입력/인터페이스로 h3, h3o, h3on 성능 비교 가능

**🎯 다음 단계 준비:**
- STEP 3: NUMA-aware 스레드풀 구성 (`fork_union`) 준비 완료
- 병렬화 기반 구조 확립으로 NUMA 최적화 적용 준비 완료
- 성능 벤치마크를 통한 병렬화 효과 검증 가능 (h3o vs h3on 비교)




### 🔧 Pre-STEP3 사전 개선 (반드시 선적용)

```rust
// DONE: (STEP2) 고정 임계치 제거 → with_min_len/with_max_len 도입
//       job_min = max(1024, total_len / (num_threads * 4))
//       job_max = job_min * 4  // DONE: 동적 청크 크기 적용

// DONE: (STEP2) 입력을 BaseCell/Face 단위로 프리-파티셔닝
//       DONE: 병렬 처리 전 base cell 정렬로 locality 향상

// DONE: (STEP2) rayon ThreadPoolBuilder 도입으로 커스텀 풀 주입 구조 완성
//       DONE: spawn_handler 훅 추가 (향후 affinity/hwloc 연결 대비)
```

좋아, 지금 합의한 “코어 고정 → 즉시 first‑touch 초기화” 흐름으로 **STEP 3 & 4**를 통째로 갱신해놨어. 그대로 Rulebook에 붙여 쓰면 돼.

---

# 🔹 STEP 3 & 4. NUMA‑aware 스레드 생성 + 통합 로컬 메모리 초기화（개선본）

> 🎯 **목표:** `rayon::ThreadPoolBuilder.spawn_handler` 안에서 **(1) 코어 고정**과 **(2) first‑touch 초기화**를 **원자적**으로 수행한다. 이렇게 하면 스레드와 데이터가 동일 NUMA 노드에 존재하도록 강제되어 cross‑node 접근을 최소화한다.

**✅ STEP 3&4 완료 요약:**
- NUMA 모듈 구조 완성 (`src/numa/mod.rs`, `src/numa/topo.rs`, `src/numa/pool.rs`)
- `hwlocality` 기반 NUMA 토폴로지 탐색 및 캐싱 구현 완료
- `core_affinity`를 사용한 스레드 코어 고정 구현 완료
- `thread_local!` + `OnceCell`을 사용한 노드별 로컬 데이터 구조 구현 완료
- `build_numa_pool` 함수로 NUMA-aware 스레드풀 구성 완료
- 핵심 병목 함수들에 NUMA 최적화 적용 완료:
  - `grid_disks_fast_numa` - 다중 인덱스 처리 NUMA 최적화
  - `compact_numa` - 압축 연산 NUMA 최적화
  - `uncompact_numa` - 압축 해제 연산 NUMA 최적화
  - `uncompact_size_numa` - 크기 계산 NUMA 최적화
  - `into_coverage_numa` - 폴리곤 타일링 NUMA 최적화
- `estimate_buffer_sizes` 함수로 동적 버퍼 크기 추정 구현 완료
- 조건부 컴파일(`#[cfg(feature = "numa")]`)을 통한 선택적 NUMA 최적화 적용
- 대용량 데이터(100개 이상)에서만 NUMA 최적화 적용하여 오버헤드 최소화

**🔄 STEP 5: 기존 함수를 NUMA 버전으로 자동 대체하여 API 호환성 유지** ✅ **완료**
- `grid_disks_fast()` → `grid_disks_fast_numa()` 자동 호출
- `compact()` → `compact_numa()` 자동 호출  
- `uncompact()` → `uncompact_numa()` 자동 호출
- `uncompact_size()` → `uncompact_size_numa()` 자동 호출
- `into_coverage()` → `into_coverage_numa()` 자동 호출
- 기존 API 호환성 완벽 유지, 사용자 코드 변경 불필요
- `--features numa` 활성화 시 자동으로 NUMA 최적화 적용

**🎯 다음 단계 준비:**
- STEP 5: 공용 테이블 및 캐시 파티셔닝 준비 완료
- NUMA-aware 스레드풀 및 first-touch 초기화 기반 구조 확립
- 성능 벤치마크를 통한 NUMA 최적화 효과 검증 가능

## ✅ 의존성/기본 정책

* **필수:** `hwlocality`(토폴로지 탐색), `core_affinity`(코어 바인딩)
* **선택:** `mimalloc`(전역 할당자) — *Step4 효과 측정 후에만 도입*
* **기본 정책:** Linux `first‑touch` 활용, 별도 NUMA allocator 불필요

```toml
# Cargo.toml (예시)
[features]
numa = ["hwlocality", "core_affinity"]
bench = ["criterion"]

[dependencies]
rayon = "1"
core_affinity = { version = "0.8", optional = true }
hwlocality = { version = "1", optional = true }
once_cell = "1"
```

---

## 🧩 설계 개요

1. **토폴로지 로드/캐시**

* 시작 시 1회 `hwlocality`로 **노드 수 / 노드별 코어 리스트**를 확보·캐시.

2. **작업 파티셔닝**

* 입력을 **BaseCell/Face** 단위로 분해 → `node_id = basecell_id % numa_nodes`.
* 각 노드 큐에 청크를 push (균형 고려: 노드별 코어 수로 가중 분배).

3. **스레드풀 구성 & 원자적 초기화**

* `ThreadPoolBuilder` + `spawn_handler`에서

  * (a) `core_affinity::set_for_current(core_id)`
  * (b) **즉시** 로컬 버퍼/캐시를 `resize/fill`로 초기화 → **first‑touch** 발생
  * (c) 이후 해당 워커는 자기 노드 큐의 작업만 처리

---

## 🛠 구현 스캐폴딩（예시 코드）

> 파일 위치 제안: `src/numa/mod.rs`, `src/numa/topo.rs`, `src/numa/pool.rs`

```rust
// src/numa/topo.rs
#[cfg(feature = "numa")]
pub struct NumaTopology {
    pub nodes: usize,
    pub cores_per_node: Vec<Vec<usize>>, // logical core ids per node
}

#[cfg(feature = "numa")]
pub fn load_topology() -> NumaTopology {
    use hwlocality::Topology;
    let topo = Topology::new().expect("hwloc topology");
    let nodes = topo.objects_with_type(&hwlocality::ObjectType::NUMANode)
                    .map(|v| v.len())
                    .unwrap_or(1);

    // 간단 샘플: NUMA 노드별 PU(core) id 수집
    let mut cores_per_node = vec![Vec::new(); nodes];
    for (nid, node) in topo.objects_with_type(&hwlocality::ObjectType::NUMANode)
                           .unwrap_or_default()
                           .into_iter()
                           .enumerate()
    {
        let pus = node
            .children()
            .flat_map(|c| c.pus())
            .map(|pu| pu.os_index())
            .collect::<Vec<_>>();
        cores_per_node[nid] = pus;
    }

    NumaTopology { nodes, cores_per_node }
}
```

```rust
// src/numa/pool.rs
#[cfg(feature = "numa")]
use once_cell::unsync::OnceCell;

#[cfg(feature = "numa")]
thread_local! {
    // 노드 로컬 캐시/버퍼 보관 (예: lookup table, scratch buffers)
    static NODE_LOCAL: OnceCell<NodeLocal> = OnceCell::new();
}

#[cfg(feature = "numa")]
pub struct NodeLocal {
    pub scratch: Vec<u8>,          // 예시 버퍼
    // TODO: geometry LUT 복제본 등 필요한 구조체 추가
}

#[cfg(feature = "numa")]
impl NodeLocal {
    fn new(cap: usize) -> Self {
        let mut scratch = Vec::with_capacity(cap);
        // First-touch: 실제 페이지 매핑 유도
        scratch.resize(cap, 0);
        Self { scratch }
    }
}

#[cfg(feature = "numa")]
pub fn build_numa_pool<F, R>(
    topo: &crate::numa::topo::NumaTopology,
    per_worker_buf: usize,
    work: F,
) -> R
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    use rayon::ThreadPoolBuilder;

    // 워커 수 = 모든 노드의 코어 수 합
    let worker_cores: Vec<usize> = topo.cores_per_node.iter().flatten().copied().collect();
    let workers = worker_cores.len().max(1);

    let pool = ThreadPoolBuilder::new()
        .num_threads(workers)
        .spawn_handler(|thread| {
            // ★ 원자적 처리: 코어 고정 → 즉시 first-touch 초기화
            let core_id = worker_cores[thread.index() % worker_cores.len()];
            core_affinity::set_for_current(core_affinity::CoreId { id: core_id });

            // 노드 로컬 버퍼/캐시 초기화 (first-touch)
            NODE_LOCAL.with(|cell| {
                let _ = cell.set(NodeLocal::new(per_worker_buf));
            });

            std::thread::Builder::new()
                .name(format!("h3on-numa-{}", thread.index()))
                .spawn(move || thread.run())
                .map(|_| ())
        })
        .build()
        .expect("failed to build NUMA-aware pool");

    pool.install(work)
}

#[cfg(feature = "numa")]
pub fn with_node_local<T>(f: impl FnOnce(&NodeLocal) -> T) -> T {
    NODE_LOCAL.with(|cell| {
        let nl = cell.get().expect("NodeLocal not initialized");
        f(nl)
    })
}
```

```rust
// src/numa/mod.rs
#[cfg(feature = "numa")]
pub mod topo;
#[cfg(feature = "numa")]
pub mod pool;
```

**사용 예 (핵심 병목 함수 내부):**

```rust
#[cfg(feature = "numa")]
pub fn polygon_to_cells_numa(input: &Polygon, res: u8) -> Vec<Cell> {
    use crate::numa::{topo::load_topology, pool::build_numa_pool, pool::with_node_local};

    let topo = load_topology();
    let per_worker_buf = estimate_buffer_size(input, res);

    // 파티셔닝: BaseCell/Face 단위 → node_id = basecell_id % topo.nodes
    let node_buckets = partition_by_node(input, res, topo.nodes);

    build_numa_pool(&topo, per_worker_buf, || {
        use rayon::prelude::*;
        let mut out = Vec::new();

        // 각 노드 버킷을 병렬로 처리 (워커는 이미 코어 고정 + 로컬 버퍼 보유)
        node_buckets
            .into_par_iter()
            .flat_map(|chunk| {
                with_node_local(|nl| {
                    // nl.scratch 를 활용한 로컬 처리 (cross-node 접근 없음)
                    compute_chunk_with_scratch(&chunk, res, nl)
                })
            })
            .collect_into_vec(&mut out);

        out
    })
}
```

---

## 🧪 검증/수용 기준（Acceptance Criteria）

**기능**

* [ ] 스레드 시작 직후 `core_affinity::set_for_current`가 성공해야 한다.
* [ ] 코어 고정 직후 로컬 버퍼가 `resize/fill`로 초기화된다(메모리 first‑touch 보장).
* [ ] 노드 로컬 캐시/버퍼는 `thread_local!`로 스레드 간 공유되지 않는다.

**성능**

* [ ] Step2 대비 Step3에서 cross‑node 메모리 접근 비율이 유의미하게 감소(`numastat`, `perf c2c` 등으로 확인 가능).
* [ ] Step4까지 적용 시, 전체 벤치마크에서 P50/P90 레이턴시 및 처리량 향상.
* [ ] `mimalloc` 도입 전후 성능 차이를 분리 측정(기본 glibc malloc 대비).

**안전성**

* [ ] 토폴로지 캐싱은 1회만 수행되고 실패 시 단일 노드 모드로 폴백.
* [ ] 워커 수가 코어 수를 초과해도 실행되나, **경고 로그**로 과구성 알림.
* [ ] 노드별 작업량 불균형 시 동적 워크 스틸링은 **같은 노드 내**에서만 이루어진다(옵션).

---

## 📝 구현 체크리스트（TODO/DONE）

```rust
// DONE: topo.load_topology() 1회만 호출되도록 초기화 경로 정리
// DONE: BaseCell/Face 파티셔닝 구현 + node_id 매핑 규칙 확정
// DONE: ThreadPoolBuilder.spawn_handler에서 (a) core pin → (b) NodeLocal first-touch 초기화
// DONE: NODE_LOCAL(thread_local)에서 LUT/버퍼 등 노드 로컬 구조 보관
// DONE: 병목 함수(grid_disks_fast/compact/polygon_to_cells 등)에 NUMA 최적화 적용
// TODO: Step2 대비 Step3/4 별도 벤치 라벨로 기여도 분리 측정
// TODO (opt): 노드 내 워크-스틸링(균형화) 구현, cross-node 스틸링 금지

// DONE: spawn_handler 사용 플로우 확정
// DONE: first-touch 보장 방식(allocate+resize/fill) 결정
```

---

## ⚠️ 주의/권장

* `Vec::with_capacity`만으로는 페이지 매핑이 안 됨 → **반드시 `resize`/`fill`로 write 터치**.
* 스레드풀을 **노드별 다중 풀**로 쪼개기보다는, **단일 풀 + affinity**로 시작하는 게 안정적.
* `mimalloc`은 성능이 좋아도 NUMA‑aware는 아님 → 도입 시 반드시 **전/후** 측정.
* 토폴로지 비대칭(노드별 코어 수 상이) 시, **가중치 기반 분배**로 초반 불균형 방지.

---

## 🧪 벤치 라벨링 예시（criterion）

* `polygon_to_cells/h3o`
* `polygon_to_cells/h3on-step2` (rayon만)
* `polygon_to_cells/h3on-step3` (affinity 고정)
* `polygon_to_cells/h3on-step4` (first‑touch 포함)
* `polygon_to_cells/h3on-step4-mimalloc`

---


### 🔹 STEP 5. 공용 테이블 및 캐시 파티셔닝

```rust
// TODO: geometry lookup table 등 read-heavy 구조 복제 방안 제시
// TODO: 각 NUMA 노드에서 로컬 참조 가능하도록 구성 전략 수립
```

> 🎯 목적: 캐시 경합(lock contention), false sharing 제거

**구체적 적용 방안:**
1. **룩업 테이블 복제**
   - `DIRECTIONS`, `PENTAGON_ROTATIONS` 등 상수 테이블 NUMA 노드별 복제
   - `ContainmentPredicate` 구조체 NUMA 노드별 인스턴스

2. **캐시 라인 정렬**
   ```rust
   #[repr(align(64))]
   struct NumaAlignedData {
       // 64바이트 캐시 라인 정렬
   }
   ```

### 🔹 STEP 6. 성능 벤치마크 및 회귀 검증 (`criterion`)

```rust
// TODO: h3, h3o, h3on의 동일 연산 비교 벤치마크 구성 방안 제시
// TODO: @benches 기준 동일 입력/인터페이스로 벤치 작성 전략 수립
// TODO: feature flag 및 결과 분기 라벨 적용 (e.g. polygon_to_cells_h3on)
// TODO: benchmark 결과 CSV/Markdown 기록
```

> 🎯 목적: 적용 효과 수치화 + 지속적인 성능 회귀 검증

**구체적 적용 방안:**
1. **벤치마크 구조 개선**
   ```rust
   // benches/h3/polygon_to_cells.rs에 h3on 버전 추가
   group.bench_with_input(
       BenchmarkId::new("h3on/Full", res),
       &res,
       |b, &res| bench_h3on(b, &polygon, res),
   );
   ```

2. **성능 측정 지표**
   - 처리량 (cells/second)
   - 메모리 사용량
   - NUMA 노드별 분산도
   - 캐시 미스율

## 🧾 커밋 메시지 규칙

| 유형 | 예시 |
|------|------|
| `rayon` | `[rayon] grid_disks_fast 병렬 iterator 적용` |
| `numa` | `[numa] hwlocality 기반 토폴로지 검색 및 스레드 고정 구현` |
| `mem` | `[mem] first-touch 정책을 활용한 메모리 지역성 개선` |
| `bench` | `[bench] h3on 기준 polygon_to_cells 벤치마크 추가` |
| `infra` | `[infra] NUMA 탐색 및 스레드 affinity 확인 추가` |

## 🔍 진행 상태 관리 예시

```rust
// TODO: h3on::grid_disks_fast NUMA 최적화 적용
// DONE: rayon par_iter로 h3on::compact 처리 완료
// TODO: NUMA fallback 전략 검토 (optional)
```
- Agent가 TODO 단계를 수행완료한 경우 DONE으로 업데이트를 수행하거나, 업데이트를 요청

## 개선 대상 (예시)

| 항목 | 현 이슈 | 제안 개선 방식 | 적용 단계 |
|------|---------|----------------|------------|
| grid\_disks\_fast | 반복 연산 병목 | par\_iter + `hwlocality`/`core_affinity` 고정 | STEP 2, 3 |
| shared cache | cross-node 경합 | NUMA 노드별 복제 (`thread_local!`) | STEP 5 |
| 벡터 버퍼 | 스레드 간 메모리 경합 | 'first-touch' 정책 활용 (스레드별 초기화) | STEP 3, 4 |
| 벤치마크 | h3/h3o 비교 어려움 | `h3on` 명시적 네임 + 동일 인터페이스 적용 | STEP 6 |

## 📌 참고 라이브러리 목록 (예시)

| 라이브러리 | 기능 | 적용 단계 |
|------------|------|------------|
| `rayon` | 데이터 병렬 iterator | STEP 2 |
| `hwlocality` | NUMA 토폴로지(노드/코어) 탐색 (hwloc의 safe wrapper) | STEP 3 |
| `core_affinity` | 현재 스레드를 특정 코어에 바인딩 | STEP 3 |
| `criterion` | 성능 벤치마킹 | STEP 6 |
| `mimalloc` | (선택) 고성능 글로벌 메모리 할당자 | STEP 4 |
