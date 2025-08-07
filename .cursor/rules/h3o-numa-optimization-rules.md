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
// TODO: polygon_to_cells 병렬화 적용
// TODO: compact 연산 병렬화 적용 및 성능 비교
```

> 🎯 목적: 병렬 구조 기반 확보, 이후 NUMA 적용의 기반 마련

**구체적 적용 방안:**
1. **`grid_disks_fast` 병렬화** (src/index/cell.rs:1152-1160)
   ```rust
   // 현재: 순차적 flat_map
   indexes.into_iter().flat_map(move |index| index.grid_disk_fast(k))
   
   // 개선: rayon par_iter 적용
   use rayon::prelude::*;
   indexes.into_par_iter().flat_map_iter(move |index| index.grid_disk_fast(k))
   ```

2. **`compact` 병렬화** (src/index/cell.rs:669-725)
   - 정렬 단계: `par_sort_unstable` 적용
   - 압축 단계: 청크 단위 병렬 처리

3. **`into_coverage` 병렬화** (src/geom/tiler.rs:153-247)
   - 외곽선 계산: 다중 폴리곤 병렬 처리
   - 내부 전파: 레이어별 병렬 처리

### 🔹 STEP 3. NUMA-aware 스레드풀 구성 (`fork_union`)

```rust
// DONE: fork_union::linux_colocated_pool() 사용 방안 제시
// DONE: NUMA 노드별 chunking 및 로컬 작업 분할 전략 수립
// TODO: 스레드 간 메모리 접근이 cross-node 되지 않도록 분리
```

> 🎯 목적: 연산을 NUMA 로컬 영역 내에서 실행, 스레드 마이그레이션 제거

**구체적 적용 방안:**
1. **NUMA-aware 스레드풀 설정**
   ```rust
   use fork_union::linux_colocated_pool;
   
   // NUMA 노드별 스레드풀 구성
   let numa_pool = linux_colocated_pool().expect("NUMA pool creation failed");
   ```

2. **데이터 분할 전략**
   - `grid_disks_fast`: 인덱스별 NUMA 노드 분산
   - `compact`: 청크 단위 NUMA 노드별 처리
   - `into_coverage`: 폴리곤별 NUMA 노드 분산

### 🔹 STEP 4. NUMA-aware 메모리 할당 (`numanji`)

```rust
// DONE: LocalAllocator를 사용해 벡터/버퍼 NUMA 노드에 고정 방안 제시
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
// DONE: geometry lookup table 등 read-heavy 구조 복제 방안 제시
// DONE: 각 NUMA 노드에서 로컬 참조 가능하도록 구성 전략 수립
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
// DONE: h3, h3o, h3on의 동일 연산 비교 벤치마크 구성 방안 제시
// DONE: @benches 기준 동일 입력/인터페이스로 벤치 작성 전략 수립
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