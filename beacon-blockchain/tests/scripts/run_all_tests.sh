#!/bin/bash
# BEACON Test Automation Script
# Runs comprehensive test suite for the BEACON blockchain platform

set -e  # Exit on any error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
export RUST_LOG=info
export DATABASE_URL=":memory:"
export JWT_SECRET="test_secret_key_for_comprehensive_testing"
export API_PORT=3001
export RATE_LIMIT_REQUESTS=1000
export RATE_LIMIT_WINDOW=60

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run a test category with timing
run_test_category() {
    local category=$1
    local command=$2
    
    print_status "Running $category tests..."
    
    start_time=$(date +%s)
    
    if eval "$command"; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        print_success "$category tests completed in ${duration}s"
        return 0
    else
        print_error "$category tests failed"
        return 1
    fi
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi
    
    if ! command -v sqlite3 &> /dev/null; then
        print_warning "SQLite3 not found. Some tests may fail."
    fi
    
    print_success "Prerequisites check completed"
}

# Function to clean up test artifacts
cleanup() {
    print_status "Cleaning up test artifacts..."
    
    # Remove test databases
    find . -name "test_*.db" -delete 2>/dev/null || true
    find . -name "*.db-wal" -delete 2>/dev/null || true
    find . -name "*.db-shm" -delete 2>/dev/null || true
    
    # Clean cargo test artifacts
    cargo clean --target-dir target/test 2>/dev/null || true
    
    print_success "Cleanup completed"
}

# Function to generate test report
generate_report() {
    local passed=$1
    local failed=$2
    local total=$((passed + failed))
    
    echo ""
    echo "==============================================="
    echo "           BEACON TEST REPORT"
    echo "==============================================="
    echo "Total test categories: $total"
    echo "Passed: $passed"
    echo "Failed: $failed"
    
    if [ $failed -eq 0 ]; then
        echo -e "Status: ${GREEN}ALL TESTS PASSED${NC}"
        echo "==============================================="
        return 0
    else
        echo -e "Status: ${RED}SOME TESTS FAILED${NC}"
        echo "==============================================="
        return 1
    fi
}

# Main test execution function
run_all_tests() {
    print_status "Starting BEACON comprehensive test suite..."
    
    local passed=0
    local failed=0
    
    # 1. Unit Tests
    if run_test_category "Unit" "cargo test --lib --workspace"; then
        ((passed++))
    else
        ((failed++))
    fi
    
    # 2. Integration Tests
    if run_test_category "Integration" "cargo test --test integration_tests"; then
        ((passed++))
    else
        ((failed++))
    fi
    
    # 3. API Tests
    if run_test_category "API" "cargo test --test api_tests"; then
        ((passed++))
    else
        ((failed++))
    fi
    
    # 4. Documentation Tests
    if run_test_category "Documentation" "cargo test --doc --workspace"; then
        ((passed++))
    else
        ((failed++))
    fi
    
    # 5. Performance Tests (if requested)
    if [ "${RUN_PERFORMANCE_TESTS:-false}" = "true" ]; then
        if run_test_category "Performance" "cargo test --release --test performance_tests"; then
            ((passed++))
        else
            ((failed++))
        fi
    fi
    
    # 6. Benchmark Tests (if requested)
    if [ "${RUN_BENCHMARKS:-false}" = "true" ]; then
        if run_test_category "Benchmarks" "cargo bench --bench api_benchmarks --bench storage_benchmarks"; then
            ((passed++))
        else
            ((failed++))
        fi
    fi
    
    generate_report $passed $failed
}

# Function to run specific test category
run_specific_test() {
    local test_type=$1
    
    case $test_type in
        "unit")
            run_test_category "Unit" "cargo test --lib --workspace"
            ;;
        "integration")
            run_test_category "Integration" "cargo test --test integration_tests"
            ;;
        "api")
            run_test_category "API" "cargo test --test api_tests"
            ;;
        "performance")
            run_test_category "Performance" "cargo test --release --test performance_tests"
            ;;
        "benchmarks")
            run_test_category "Benchmarks" "cargo bench"
            ;;
        "doc")
            run_test_category "Documentation" "cargo test --doc --workspace"
            ;;
        *)
            print_error "Unknown test type: $test_type"
            print_status "Available types: unit, integration, api, performance, benchmarks, doc"
            exit 1
            ;;
    esac
}

# Function to setup test environment
setup_test_environment() {
    print_status "Setting up test environment..."
    
    # Build all crates first
    cargo build --workspace
    
    # Create test data directory if it doesn't exist
    mkdir -p target/test-data
    
    # Set up test database if needed
    if [ "${USE_PERSISTENT_DB:-false}" = "true" ]; then
        export DATABASE_URL="sqlite:target/test-data/test_beacon.db"
    fi
    
    print_success "Test environment setup completed"
}

# Function to run tests with coverage
run_with_coverage() {
    print_status "Running tests with coverage analysis..."
    
    if command -v cargo-tarpaulin &> /dev/null; then
        cargo tarpaulin --out Html --output-dir target/coverage --workspace --timeout 300
        print_success "Coverage report generated in target/coverage/"
    else
        print_warning "cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin"
        print_status "Running tests without coverage..."
        run_all_tests
    fi
}

# Function to display help
show_help() {
    echo "BEACON Test Automation Script"
    echo ""
    echo "Usage: $0 [OPTIONS] [TEST_TYPE]"
    echo ""
    echo "OPTIONS:"
    echo "  -h, --help              Show this help message"
    echo "  -c, --coverage          Run tests with coverage analysis"
    echo "  -p, --performance       Include performance tests"
    echo "  -b, --benchmarks        Include benchmark tests"
    echo "  --clean                 Clean test artifacts before running"
    echo "  --persistent-db         Use persistent test database"
    echo "  --verbose               Run tests with verbose output"
    echo ""
    echo "TEST_TYPE:"
    echo "  unit                    Run only unit tests"
    echo "  integration            Run only integration tests"
    echo "  api                    Run only API tests"
    echo "  performance            Run only performance tests"
    echo "  benchmarks             Run only benchmark tests"
    echo "  doc                    Run only documentation tests"
    echo "  (no type)              Run all applicable tests"
    echo ""
    echo "EXAMPLES:"
    echo "  $0                      # Run all tests"
    echo "  $0 unit                 # Run only unit tests"
    echo "  $0 -c                   # Run all tests with coverage"
    echo "  $0 -p -b               # Run all tests including performance and benchmarks"
    echo "  $0 --clean api         # Clean and run API tests"
}

# Parse command line arguments
COVERAGE=false
CLEAN=false
VERBOSE=false
TEST_TYPE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -c|--coverage)
            COVERAGE=true
            shift
            ;;
        -p|--performance)
            export RUN_PERFORMANCE_TESTS=true
            shift
            ;;
        -b|--benchmarks)
            export RUN_BENCHMARKS=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --persistent-db)
            export USE_PERSISTENT_DB=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            export RUST_LOG=debug
            shift
            ;;
        unit|integration|api|performance|benchmarks|doc)
            TEST_TYPE=$1
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Set verbose cargo output if requested
if [ "$VERBOSE" = true ]; then
    export CARGO_TERM_VERBOSE=true
fi

# Main execution
main() {
    echo "🧪 BEACON Blockchain Test Suite"
    echo "==============================="
    
    # Check prerequisites
    check_prerequisites
    
    # Clean if requested
    if [ "$CLEAN" = true ]; then
        cleanup
    fi
    
    # Setup test environment
    setup_test_environment
    
    # Run tests based on options
    if [ "$COVERAGE" = true ]; then
        run_with_coverage
    elif [ -n "$TEST_TYPE" ]; then
        run_specific_test "$TEST_TYPE"
    else
        run_all_tests
    fi
    
    # Final cleanup
    if [ "$CLEAN" = true ]; then
        cleanup
    fi
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
