#!/usr/bin/env bash
set -euo pipefail

# runzip.sh 상단 근처에 추가
TS="$(date +%Y%m%d_%H%M%S)"
export NUMA_MON_TAG="H3ON_BENCH"   # ★ 벤치 태그 (자식 프로세스까지 상속)


# === 설정값 ===
ARCHIVE_PREFIX="criterion"
ARCHIVE_DIR="."                # 압축파일 저장 경로(현재 디렉터리)
CRITERION_DIR="target/criterion"
PATTERN="${ARCHIVE_PREFIX}_*.tar.gz"   # 기존 압축파일 삭제 패턴
TS="$(date +%Y%m%d_%H%M%S)"
ARCHIVE_NAME="${ARCHIVE_PREFIX}_${TS}.tar.gz"
ARCHIVE_PATH="${ARCHIVE_DIR%/}/${ARCHIVE_NAME}"
BENCH_GROUP="NUMA_Parallel_Performance"

echo "[1/3] 기존 압축파일 삭제(${PATTERN})"
find "${ARCHIVE_DIR}" -maxdepth 1 -type f -name "${PATTERN}" -print -delete || true
rm -rf ${CRITERION_DIR}

echo "[2/3] cargo bench 실행 (${BENCH_GROUP})"
# 필요 시 --bench-filter 사용 가능. 현재는 명령 그대로 실행
#cargo bench --features numa -- areNeighborCells
cargo bench --features numa -- ${BENCH_GROUP}

# 결과 디렉터리 존재 확인
if [[ ! -d "${CRITERION_DIR}" ]]; then
  echo "에러: ${CRITERION_DIR} 디렉터리를 찾지 못했습니다. benchmark가 결과를 생성했는지 확인하세요." >&2
  exit 1
fi

echo "[3/3] Criterion 결과 압축 → ${ARCHIVE_PATH}"
# pigz가 있으면 병렬 gzip 사용, 없으면 기본 gzip
if command -v pigz >/dev/null 2>&1; then
  tar -I pigz -cvf "${ARCHIVE_PATH}" "${CRITERION_DIR}"
else
  tar -czvf "${ARCHIVE_PATH}" "${CRITERION_DIR}"
fi

echo "완료: ${ARCHIVE_PATH}"
# 원본 삭제를 원하면 아래 주석 해제
# echo "원본 Criterion 디렉터리 삭제"
# rm -rf "${CRITERION_DIR}"

