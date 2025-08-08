# 🧠 `h3on` NUMA 최적화 Rulebook

## 🎯 프로젝트 목표

> `h3on`은 [HydroniumLabs/h3o](https://github.com/HydroniumLabs/h3o)의 fork로, NUMA 기반 멀티코어 환경에서 **대규모 공간 연산을 병렬화하고, 메모리 locality를 개선**하여 성능을 4~7배 향상시키는 것을 목표로 합니다.

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
// TODO: (STEP2) 고정 임계치 제거 → with_min_len/with_max_len 도입
//  - job_min = max(1024, total_len / (num_threads * 4))
//  - job_max = job_min * 4

// TODO: (STEP2) 입력을 BaseCell/Face 단위로 프리-파티셔닝
//  - 이후 NUMA 노드 매핑 시 cross-node 접근 감소

// TODO: (STEP2) rayon ThreadPoolBuilder 도입으로 커스텀 풀 주입 구조 완성
//  - spawn_handler 훅 제공 (향후 affinity/hwloc 연결)
```

### 🔹 STEP 3. NUMA-aware 스레드/메모리 (개선안)

- **스레드풀/어피니티 (2트랙)**  
  - 기본(안정): `rayon` + (`affinity` | `hwloc`)로 코어/NUMA 노드 바인딩  
  - 옵션(실험): `feature("numa-fork-union")` 활성 시 `fork_union` 사용
- **데이터↔노드 매핑**: 청크 *i* → NUMA 노드 *(i % N)*, BaseCell 단위 청크 유지
- **메모리 로컬리티**: `feature("numanji")` 우선, 폴백 또는 대안으로 `mimalloc` 글로벌 할당자

```rust
// TODO: 두 트랙 병행 적용
// 기본(안정): rayon + (affinity | hwloc) 로 코어/노드 바인딩
// 옵션(실험): feature("numa-fork-union") 활성화 시 fork_union 사용

// TODO: 데이터-노드 매핑 규칙
// - 청크 i -> NUMA node (i % N)
// - BaseCell 단위 청크 유지로 경계 교차 최소화

// TODO: 메모리 로컬리티
// - feature("numanji") 활성 시 LocalAllocator 우선, 실패 시 폴백
// - 또는 mimalloc 글로벌 할당자 채택(옵션)으로 NUMA-aware 할당
```

### 🔹 STEP 4. NUMA-aware 메모리 할당 (`numanji`)

```rust
// TODO: LocalAllocator를 사용해 벡터/버퍼 NUMA 노드에 고정 방안 제시
// TODO: 연산 중 메모리 locality 측정 및 비교
```

> 🎯 목적: cross-node memory access 방지, 캐시 활용도 향상

**구체적 적용 방안:**
1. **메모리 할당 최적화**
   ```rust
   use numanji::LocalAllocator;
   
   // NUMA 노드별 로컬 할당자 사용
   let local_alloc = LocalAllocator::new(numa_node_id);
   let mut cells = Vec::with_capacity_in(capacity, &local_alloc);
   ```

2. **데이터 구조 최적화**
   - `HashSet` 대신 NUMA-aware 해시맵 사용
   - 스크래치패드 메모리 로컬 할당

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
| `numa` | `[numa] fork_union 기반 NUMA-aware 스레드풀 구현` |
| `mem` | `[mem] numanji 메모리 할당 적용` |
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
| grid_disks_fast | 반복 연산 병목 | par_iter + NUMA 스레드 고정 | STEP 2, 3 |
| shared cache | cross-node 경합 | NUMA 노드별 복제 | STEP 5 |
| 벡터 버퍼 | 스레드 간 메모리 경합 | LocalAllocator로 고정 | STEP 4 |
| 벤치마크 | h3/h3o 비교 어려움 | `h3on` 명시적 네임 + 동일 인터페이스 적용 | STEP 6 |

## 📌 참고 라이브러리 목록

| 라이브러리 | 기능 | 적용 단계 |
|------------|------|------------|
| `rayon` | 데이터 병렬 iterator | STEP 2 |
| `