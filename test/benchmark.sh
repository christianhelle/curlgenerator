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
