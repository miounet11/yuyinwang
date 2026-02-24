#!/bin/bash

# Recording King Test Runner
# Comprehensive test execution script with reporting

set -e

COLOR_GREEN='\033[0;32m'
COLOR_BLUE='\033[0;34m'
COLOR_YELLOW='\033[1;33m'
COLOR_RED='\033[0;31m'
COLOR_RESET='\033[0m'

echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
echo -e "${COLOR_BLUE}Recording King Test Suite${COLOR_RESET}"
echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
echo ""

# Function to run tests with a label
run_test_suite() {
    local label=$1
    local command=$2

    echo -e "${COLOR_YELLOW}Running: ${label}${COLOR_RESET}"
    if eval "$command"; then
        echo -e "${COLOR_GREEN}✓ ${label} passed${COLOR_RESET}"
        echo ""
        return 0
    else
        echo -e "${COLOR_RED}✗ ${label} failed${COLOR_RESET}"
        echo ""
        return 1
    fi
}

# Track failures
FAILED_TESTS=()

# 1. Unit Tests - Commands
echo -e "${COLOR_BLUE}=== Unit Tests: Commands ===${COLOR_RESET}"
run_test_suite "Settings Tests" "cargo test --lib settings_test" || FAILED_TESTS+=("Settings")
run_test_suite "History Tests" "cargo test --lib history_test" || FAILED_TESTS+=("History")
run_test_suite "Injection Tests" "cargo test --lib injection_test" || FAILED_TESTS+=("Injection")
run_test_suite "Models Tests" "cargo test --lib models_test" || FAILED_TESTS+=("Models")
run_test_suite "Recording Tests" "cargo test --lib recording_test" || FAILED_TESTS+=("Recording")
run_test_suite "Quick Input Tests" "cargo test --lib quick_input_test" || FAILED_TESTS+=("QuickInput")

# 2. Unit Tests - Core
echo -e "${COLOR_BLUE}=== Unit Tests: Core ===${COLOR_RESET}"
run_test_suite "Types Tests" "cargo test --lib types_test" || FAILED_TESTS+=("Types")

# 3. Unit Tests - Services
echo -e "${COLOR_BLUE}=== Unit Tests: Services ===${COLOR_RESET}"
run_test_suite "Database Tests" "cargo test --lib database::tests" || FAILED_TESTS+=("Database")

# 4. Integration Tests
echo -e "${COLOR_BLUE}=== Integration Tests ===${COLOR_RESET}"
run_test_suite "Integration Tests" "cargo test --test integration_tests" || FAILED_TESTS+=("Integration")

# 5. Test Utilities
echo -e "${COLOR_BLUE}=== Test Utilities ===${COLOR_RESET}"
run_test_suite "Test Utils" "cargo test --test test_utils" || FAILED_TESTS+=("TestUtils")

# Summary
echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"
echo -e "${COLOR_BLUE}Test Summary${COLOR_RESET}"
echo -e "${COLOR_BLUE}========================================${COLOR_RESET}"

if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
    echo -e "${COLOR_GREEN}All tests passed! ✓${COLOR_RESET}"
    exit 0
else
    echo -e "${COLOR_RED}Failed test suites:${COLOR_RESET}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "${COLOR_RED}  - $test${COLOR_RESET}"
    done
    exit 1
fi
