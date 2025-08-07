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

## 🪜 단계별 TODO (병렬화 → NUMA-aware 순서)

### 🔹 STEP 0. NUMA 적용 대상 분석 및 전략 분류

```rust
// TODO: 병렬화 및 NUMA 적용 타겟 함수 목록 작성 (e.g. grid_disks_fast, compact, polygon_to_cells)
// TODO: 연산별 병렬화/NUMA 적용 가능성 평가 (독립성, 데이터 locality)
// TODO: 각 함수의 데이터 접근 패턴 정리 및 전략 분류
// TODO: h3o-numa-optimization-rules.md 업데이트
```

> 🎯 목적: 성능 병목 지점 우선순위 지정, NUMA 전략 결정 기준 수립

### 🔹 STEP 1. 병렬화 구조 적용 (`rayon`)

```rust
// TODO: par_iter를 사용한 병렬 처리 도입 (e.g. grid_disks_fast)
// TODO: polygon_to_cells 병렬화 적용
// TODO: compact 연산 병렬화 적용 및 성능 비교
```

> 🎯 목적: 병렬 구조 기반 확보, 이후 NUMA 적용의 기반 마련

### 🔹 STEP 2. NUMA-aware 스레드풀 구성 (`fork_union`)

```rust
// TODO: fork_union::linux_colocated_pool() 사용
// TODO: NUMA 노드별 chunking 및 로컬 작업 분할
// TODO: 스레드 간 메모리 접근이 cross-node 되지 않도록 분리
```

> 🎯 목적: 연산을 NUMA 로컬 영역 내에서 실행, 스레드 마이그레이션 제거

### 🔹 STEP 3. NUMA-aware 메모리 할당 (`numanji`)

```rust
// TODO: LocalAllocator를 사용해 벡터/버퍼 NUMA 노드에 고정
// TODO: 연산 중 메모리 locality 측정 및 비교
```

> 🎯 목적: cross-node memory access 방지, 캐시 활용도 향상

### 🔹 STEP 4. 공용 테이블 및 캐시 파티셔닝

```rust
// TODO: geometry lookup table 등 read-heavy 구조 복제
// TODO: 각 NUMA 노드에서 로컬 참조 가능하도록 구성
```

> 🎯 목적: 캐시 경합(lock contention), false sharing 제거

### 🔹 STEP 5. 성능 벤치마크 및 회귀 검증 (`criterion`)

```rust
// TODO: h3, h3o, h3on의 동일 연산 비교 벤치마크 구성
// TODO: @benches 기준 동일 입력/인터페이스로 벤치 작성
// TODO: feature flag 및 결과 분기 라벨 적용 (e.g. polygon_to_cells_h3on)
// TODO: benchmark 결과 CSV/Markdown 기록
```

> 🎯 목적: 적용 효과 수치화 + 지속적인 성능 회귀 검증

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
| grid_disks_fast | 반복 연산 병목 | par_iter + NUMA 스레드 고정 | STEP 1, 2 |
| shared cache | cross-node 경합 | NUMA 노드별 복제 | STEP 4 |
| 벡터 버퍼 | 스레드 간 메모리 경합 | LocalAllocator로 고정 | STEP 3 |
| 벤치마크 | h3/h3o 비교 어려움 | `h3on` 명시적 네임 + 동일 인터페이스 적용 | STEP 5 |

## 📌 참고 라이브러리 목록

| 라이브러리 | 기능 | 적용 단계 |
|------------|------|------------|
| `rayon` | 데이터 병렬 iterator | STEP 1 |
| `fork_union` | NUMA-aware 스레드풀 | STEP 2 |
| `numanji` | NUMA 고정 메모리 할당 | STEP 3 |
| `hwlocality` | NUMA 토폴로지 탐색 | STEP 0, 2 |
| `criterion` | 벤치마크 및 회귀 검증 | STEP 5 |
| `numactl` / `libnuma` | 실험 환경 설정 도구 | STEP 0, 5 |
| `crossbeam`, `parking_lot` | 고성능 동시성 제어 (optional) | STEP 2, 4 |

## ✅ 최종 목표 정리

> `h3on`은 `grid_disks_fast`, `compact`, `polygon_to_cells`를 중심으로 NUMA-aware 최적화를 적용하여,  
> 대규모 데이터셋 기준으로 기존 h3/h3o 대비 **4~7배 이상의 성능 향상**을 목표로 합니다.