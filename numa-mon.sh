#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-H3ON_BENCH}"
INTERVAL="${2:-1}"
HAS_P=0
if numastat --help 2>/dev/null | grep -q "\-p"; then HAS_P=1; fi

build_cpu2node() {
  if command -v lscpu >/dev/null 2>&1; then
    lscpu -e=CPU,NODE,CORE,SOCKET | sort -n > /tmp/cpu2node.tsv
  else
    : > /tmp/cpu2node.tsv
  fi
}
build_cpu2node

find_tagged_pid() {
  grep -Zsl "NUMA_MON_TAG=${TAG}" /proc/*/environ 2>/dev/null \
    | sed 's#/proc/##; s#/environ##' \
    | xargs -r ps -o pid=,pcpu= --no-headers -p \
    | sort -k2 -nr | awk 'NR==1{print $1}'
}

curr_pid=""
trap 'echo; echo "[*] exit"; exit 0' INT TERM

while :; do
  new_pid="$(find_tagged_pid || true)"

  if [[ "$new_pid" != "$curr_pid" ]]; then
    if [[ -n "$new_pid" ]]; then
      echo "[*] Tracking PID: $new_pid"
    elif [[ -n "$curr_pid" ]]; then
      echo "[*] Target process ended: $curr_pid"
    fi
    curr_pid="$new_pid"
  fi

  clear
  date
  echo "=== System NUMA (numastat -n) ==="
  numastat -n 2>/dev/null | sed -n '1,12p' || echo "(numastat not available?)"

  if [[ -n "$curr_pid" && "$HAS_P" -eq 1 ]] && kill -0 "$curr_pid" 2>/dev/null; then
    echo
    echo "=== Process NUMA (numastat -p $curr_pid) ==="
    numastat -p "$curr_pid" 2>/dev/null | sed -n '1,25p' || echo "(numastat -p failed)"

    echo
    echo "=== Threads -> CPU -> NODE (top 20 by CPU) ==="
    if [[ -s /tmp/cpu2node.tsv ]]; then
      ps -Lp "$curr_pid" -o tid=,psr=,pcpu=,comm= 2>/dev/null \
        | sort -k3 -nr | head -20 \
        | awk 'NR==FNR{m[$1]=$2;next}{printf("%-8s cpu=%-4s node=%-3s pcpu=%-6s %s\n",$1,$2,($2 in m?m[$2]:"?"),$3,$4)}' /tmp/cpu2node.tsv -
    else
      ps -Lp "$curr_pid" -o tid=,psr=,pcpu=,comm= 2>/dev/null \
        | sort -k3 -nr | head -20
    fi
  else
    echo
    echo "Waiting for process with ENV '${TAG}'..."
  fi

  sleep "$INTERVAL"
done

