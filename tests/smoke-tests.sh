#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

# Detect script location and set up paths
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

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

echo -e "${CYAN}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║          cURL Generator - Smoke Test Suite                ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Verify binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}✗ Binary not found: $BINARY${NC}"
    echo -e "${YELLOW}  Building release binary...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    fi
fi

echo -e "${GRAY}Project Root: $PROJECT_ROOT${NC}"
echo -e "${GRAY}Binary: $BINARY${NC}"
echo -e "${GRAY}Test Date: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
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
    
    if ! $BINARY "$spec_path" --output "$ps_output" --skip-validation > /dev/null 2>&1; then
        echo -e "  ${RED}✗ $test_name (PowerShell)${NC}"
        return 1  # FAIL
    fi
    
    local ps_count=$(find "$ps_output" -name "*.ps1" 2>/dev/null | wc -l)
    if [ "$ps_count" -eq 0 ]; then
        echo -e "  ${RED}✗ $test_name (PowerShell - no files)${NC}"
        return 1  # FAIL
    fi
    
    # Test Bash generation
    local sh_output="$output_dir/sh"
    mkdir -p "$sh_output"
    
    if ! $BINARY "$spec_path" --output "$sh_output" --skip-validation --bash > /dev/null 2>&1; then
        echo -e "  ${RED}✗ $test_name (Bash)${NC}"
        return 1  # FAIL
    fi
    
    local sh_count=$(find "$sh_output" -name "*.sh" 2>/dev/null | wc -l)
    if [ "$sh_count" -eq 0 ]; then
        echo -e "  ${RED}✗ $test_name (Bash - no files)${NC}"
        return 1  # FAIL
    fi
    
    echo -e "  ${GREEN}✓ $test_name${NC}${GRAY} ($ps_count PS, $sh_count SH)${NC}"
    return 0  # PASS
}

# Create output directory
OUTPUT_BASE="$SCRIPT_DIR/output"
rm -rf "$OUTPUT_BASE"
mkdir -p "$OUTPUT_BASE"

# OpenAPI v2.0 specs
echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}  OpenAPI v2.0${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

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
echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}  OpenAPI v3.0${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

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
echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}  OpenAPI v3.1${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

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

echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                      Test Summary                          ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"

echo -e "\nTotal Tests:    $TOTAL_TESTS"
echo -n "Passed:         "
echo -e "${GREEN}$PASSED_TESTS${NC}"
echo -n "Failed:         "
if [ "$FAILED_TESTS" -gt 0 ]; then
    echo -e "${RED}$FAILED_TESTS${NC}"
else
    echo -e "${GREEN}$FAILED_TESTS${NC}"
fi
echo -e "${YELLOW}Skipped:        $SKIPPED_TESTS${NC}"
echo "Duration:       ${DURATION}s"

if [ "$FAILED_TESTS" -gt 0 ]; then
    echo -e "\n${RED}✗ Some tests failed${NC}"
    exit 1
else
    echo -e "\n${GREEN}✓ All tests passed!${NC}"
    exit 0
fi
