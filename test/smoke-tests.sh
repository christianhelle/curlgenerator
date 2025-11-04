#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

# Detect script location and set up paths
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Set default binary path based on detected project root
if [ -z "$1" ]; then
    BINARY="$PROJECT_ROOT/target/release/curlgenerator"
else
    BINARY="$1"
fi

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0
START_TIME=$(date +%s)

printf "${CYAN}\n"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║          cURL Generator - Smoke Test Suite                ║"
echo "╚════════════════════════════════════════════════════════════╝"
printf "${NC}\n"

# Verify binary exists
if [ ! -f "$BINARY" ]; then
    printf "${RED}✗ Binary not found: $BINARY${NC}\n"
    printf "${YELLOW}  Building release binary...${NC}\n"
    cd "$PROJECT_ROOT"
    cargo build --release
    if [ $? -ne 0 ]; then
        printf "${RED}✗ Build failed${NC}\n"
        exit 1
    fi
fi

printf "${GRAY}Project Root: $PROJECT_ROOT${NC}\n"
printf "${GRAY}Binary: $BINARY${NC}\n"
printf "${GRAY}Test Date: $(date '+%Y-%m-%d %H:%M:%S')${NC}\n"
echo ""

test_generation() {
    local version=$1
    local spec_name=$2
    local format=$3
    local output_dir=$4
    
    local spec_path="$SCRIPT_DIR/openapi/$version/$spec_name.$format"
    
    if [ ! -f "$spec_path" ]; then
        return 2  # SKIP
    fi
    
    local test_name="$version/$spec_name.$format"
    
    # Test PowerShell generation
    local ps_output="$output_dir/ps"
    mkdir -p "$ps_output"
    
    ps_start=$(date +%s%N 2>/dev/null || date +%s000000000)
    if ! $BINARY "$spec_path" --output "$ps_output" --skip-validation > /dev/null 2>&1; then
        printf "  ${RED}✗ $test_name (PowerShell)${NC}\n"
        return 1  # FAIL
    fi
    ps_end=$(date +%s%N 2>/dev/null || date +%s000000000)
    ps_duration=$(( (ps_end - ps_start) / 1000000 ))
    
    local ps_count=$(find "$ps_output" -name "*.ps1" 2>/dev/null | wc -l)
    if [ "$ps_count" -eq 0 ]; then
        printf "  ${RED}✗ $test_name (PowerShell - no files)${NC}\n"
        return 1  # FAIL
    fi
    
    # Test Bash generation
    local sh_output="$output_dir/sh"
    mkdir -p "$sh_output"
    
    sh_start=$(date +%s%N 2>/dev/null || date +%s000000000)
    if ! $BINARY "$spec_path" --output "$sh_output" --skip-validation --bash > /dev/null 2>&1; then
        printf "  ${RED}✗ $test_name (Bash)${NC}\n"
        return 1  # FAIL
    fi
    sh_end=$(date +%s%N 2>/dev/null || date +%s000000000)
    sh_duration=$(( (sh_end - sh_start) / 1000000 ))
    
    local sh_count=$(find "$sh_output" -name "*.sh" 2>/dev/null | wc -l)
    if [ "$sh_count" -eq 0 ]; then
        printf "  ${RED}✗ $test_name (Bash - no files)${NC}\n"
        return 1  # FAIL
    fi
    
    avg_duration=$(( (ps_duration + sh_duration) / 2 ))
    
    printf "  ${GREEN}✓ $test_name${NC}${GRAY} ($ps_count PS, $sh_count SH) ${NC}${CYAN}${avg_duration}ms${NC}\n"
    return 0  # PASS
}

# Create output directory
OUTPUT_BASE="$SCRIPT_DIR/output"
rm -rf "$OUTPUT_BASE"
mkdir -p "$OUTPUT_BASE"

# OpenAPI v2.0 specs
printf "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
printf "${CYAN}  OpenAPI v2.0${NC}\n"
printf "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

for spec in api-with-examples petstore petstore-expanded petstore-minimal petstore-simple petstore-with-external-docs uber; do
    for format in json yaml; do
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        test_generation "v2.0" "$spec" "$format" "$OUTPUT_BASE/v2.0/$spec/$format"
        result=$?
        case $result in
            0) PASSED_TESTS=$((PASSED_TESTS + 1)) ;;
            1) FAILED_TESTS=$((FAILED_TESTS + 1)) ;;
            2) SKIPPED_TESTS=$((SKIPPED_TESTS + 1)); TOTAL_TESTS=$((TOTAL_TESTS - 1)) ;;
        esac
    done
done

# OpenAPI v3.0 specs
printf "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
printf "${CYAN}  OpenAPI v3.0${NC}\n"
printf "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

for spec in api-with-examples callback-example hubspot-events hubspot-webhooks link-example petstore petstore-expanded uspto; do
    for format in json yaml; do
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        test_generation "v3.0" "$spec" "$format" "$OUTPUT_BASE/v3.0/$spec/$format"
        result=$?
        case $result in
            0) PASSED_TESTS=$((PASSED_TESTS + 1)) ;;
            1) FAILED_TESTS=$((FAILED_TESTS + 1)) ;;
            2) SKIPPED_TESTS=$((SKIPPED_TESTS + 1)); TOTAL_TESTS=$((TOTAL_TESTS - 1)) ;;
        esac
    done
done

# OpenAPI v3.1 specs
printf "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
printf "${CYAN}  OpenAPI v3.1${NC}\n"
printf "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

for spec in non-oauth-scopes webhook-example; do
    for format in json yaml; do
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        test_generation "v3.1" "$spec" "$format" "$OUTPUT_BASE/v3.1/$spec/$format"
        result=$?
        case $result in
            0) PASSED_TESTS=$((PASSED_TESTS + 1)) ;;
            1) FAILED_TESTS=$((FAILED_TESTS + 1)) ;;
            2) SKIPPED_TESTS=$((SKIPPED_TESTS + 1)); TOTAL_TESTS=$((TOTAL_TESTS - 1)) ;;
        esac
    done
done

# Summary
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

printf "\n${CYAN}╔════════════════════════════════════════════════════════════╗${NC}\n"
printf "${CYAN}║                      Test Summary                          ║${NC}\n"
printf "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}\n"

printf "\nTotal Tests:    $TOTAL_TESTS\n"
printf "Passed:         "
printf "${GREEN}$PASSED_TESTS${NC}\n"
printf "Failed:         "
if [ "$FAILED_TESTS" -gt 0 ]; then
    printf "${RED}$FAILED_TESTS${NC}\n"
else
    printf "${GREEN}$FAILED_TESTS${NC}\n"
fi
printf "${YELLOW}Skipped:        $SKIPPED_TESTS${NC}\n"
printf "Duration:       ${DURATION}s\n"

if [ "$FAILED_TESTS" -gt 0 ]; then
    printf "\n${RED}✗ Some tests failed${NC}\n"
    exit 1
else
    printf "\n${GREEN}✓ All tests passed!${NC}\n"
    exit 0
fi
