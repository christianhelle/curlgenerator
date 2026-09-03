#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"

BINARY="../target/release/curlgenerator"

filenames=(
  "petstore"
  "petstore-expanded"
  "petstore-minimal"
  "petstore-simple"
  "petstore-with-external-docs"
  "api-with-examples"
  "callback-example"
  "link-example"
  "uber"
  "uspto"
  "hubspot-events"
  "hubspot-webhooks"
  "non-oauth-scopes"
  "webhook-example"
  "tictactoe"
)

generate() {
  local format="$1"
  local output="$2"
  shift 2

  echo "curlgenerator ./openapi.$format --output ./Generated/$output --no-logging $*"
  if ! "$BINARY" "./openapi.$format" --output "./Generated/$output" --no-logging "$@"; then
    echo "curlgenerator failed" >&2
    exit 1
  fi
}

run_tests() {
  find . -name '*.http' -type f -delete

  echo "cargo build --release --package curlgenerator"
  if ! cargo build --release --package curlgenerator --manifest-path ../Cargo.toml; then
    echo "cargo build failed" >&2
    exit 1
  fi

  for version in "v2.0" "v3.0" "v3.1"; do
    for format in "json" "yaml"; do
      for name in "${filenames[@]}"; do
        filename="./OpenAPI/$version/$name.$format"
        if [ -f "$filename" ]; then
          echo "Testing $filename"
          cp "$filename" "./openapi.$format"
          if [ "$version" = "v3.1" ]; then
            generate "$format" "$name/$version/$format" --skip-validation
            generate "$format" "$name/$version/$format" --skip-validation --bash
          else
            generate "$format" "$name/$version/$format"
            generate "$format" "$name/$version/$format" --bash
          fi
        fi
      done
    done
  done
}

time run_tests
echo
