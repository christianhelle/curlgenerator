#!/usr/bin/env bash
# benchmark.sh - Compares the runtime of the Rust CLI and the legacy .NET CLI
# over the OpenAPI corpus.
#
# Runs both generators over every specification in test/OpenAPI, in both
# PowerShell and Bash output modes, and reports the total elapsed time for
# each. Writes a Markdown table to the GitHub step summary when running in
# Actions.

set -euo pipefail

RUST_COMMAND="../target/release/curlgenerator"
DOTNET_COMMAND="../src/dotnet/CurlGenerator/bin/Release/net8.0/curlgenerator"
RUNS=3

usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

Compares the runtime of the Rust CLI and the legacy .NET CLI over the
OpenAPI corpus.

Options:
  --rust-command <cmd>     Rust CLI to invoke (default: $RUST_COMMAND)
  --dotnet-command <cmd>   .NET CLI to invoke (default: $DOTNET_COMMAND)
  --runs <n>               How many times to repeat the corpus per implementation (default: $RUNS)
  -h, --help               Show this help message
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rust-command)
      RUST_COMMAND="$2"
      shift 2
      ;;
    --dotnet-command)
      DOTNET_COMMAND="$2"
      shift 2
      ;;
    --runs)
      RUNS="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPOSITORY="$(cd "$SCRIPT_DIR/.." && pwd)"

# Discover all OpenAPI specifications under test/OpenAPI, sorted
mapfile -t SPECIFICATIONS < <(find "$REPOSITORY/test/OpenAPI" -type f \( -name '*.json' -o -name '*.yaml' \) | sort)

if [[ ${#SPECIFICATIONS[@]} -eq 0 ]]; then
  echo "No specifications found under test/OpenAPI" >&2
  exit 1
fi

now_epoch() {
  if [[ -n "${EPOCHREALTIME:-}" ]]; then
    printf '%s' "$EPOCHREALTIME"
  else
    date +%s.%N
  fi
}

measure_generator() {
  local name="$1"
  local command="$2"
  local output
  output="$(mktemp -d -t "curlgenerator-benchmark-${name}.XXXXXX")"
  local total=0

  for (( run=1; run<=RUNS; run++ )); do
    rm -rf "${output:?}/"*
    mkdir -p "$output"

    local start end elapsed
    start="$(now_epoch)"
    for spec in "${SPECIFICATIONS[@]}"; do
      "$command" "$spec" --output "$output" --no-logging --skip-validation > /dev/null 2>&1
      "$command" "$spec" --output "$output" --no-logging --skip-validation --bash > /dev/null 2>&1
    done
    end="$(now_epoch)"
    elapsed="$(awk "BEGIN {print $end - $start}")"
    total="$(awk "BEGIN {print $total + $elapsed}")"
    printf '%s run %d of %d: %.2fs\n' "$name" "$run" "$RUNS" "$elapsed"
  done

  rm -rf "$output"
  awk "BEGIN {print $total / $RUNS}"
}
