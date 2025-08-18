# BEACON Test Automation Script for Windows PowerShell
# Runs comprehensive test suite for the BEACON blockchain platform

param(
    [switch]$Clean,
    [switch]$Verbose,
    [switch]$Coverage,
    [switch]$DebugLog,
    [switch]$NoAPI,
    [switch]$Help,
    [ValidateSet("unit", "integration", "api", "performance", "benchmarks", "doc", "all")]
    [string]$TestType = "all"
)

# Configuration
$ErrorActionPreference = "Stop"
$global:TestResults = @()
$global:StartTime = Get-Date

# Test environment configuration
$env:RUST_LOG = if ($DebugLog) { "debug" } else { "info" }
$env:DATABASE_URL = ":memory:"
$env:JWT_SECRET = "test_secret_key_for_comprehensive_testing"
$env:API_PORT = "3001"
$env:RATE_LIMIT_REQUESTS = "1000"
$env:RATE_LIMIT_WINDOW = "60"

# Color functions for output
function Write-Info { param($Message) Write-Host "[INFO] $Message" -ForegroundColor Blue }
function Write-Success { param($Message) Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "[WARNING] $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "[ERROR] $Message" -ForegroundColor Red }

# Show help information
function Show-Help {
    Write-Host "BEACON Test Suite - Windows PowerShell Edition" -ForegroundColor Cyan
    Write-Host "Usage: .\run_all_tests.ps1 [OPTIONS] [TEST_TYPE]" -ForegroundColor White
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Yellow
    Write-Host "  -Clean         Clean build artifacts before testing"
    Write-Host "  -Verbose       Enable verbose output"
    Write-Host "  -Coverage      Run tests with code coverage"
    Write-Host "  -DebugLog      Enable debug logging"
    Write-Host "  -NoAPI         Skip API server tests"
    Write-Host "  -Help          Show this help message"
    Write-Host ""
    Write-Host "Test Types:" -ForegroundColor Yellow
    Write-Host "  unit           Run only unit tests"
    Write-Host "  integration    Run only integration tests"
    Write-Host "  api            Run only API tests"
    Write-Host "  performance    Run only performance tests"
    Write-Host "  benchmarks     Run only benchmarks"
    Write-Host "  doc            Run only documentation tests"
    Write-Host "  all            Run all tests (default)"
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\run_all_tests.ps1                    # Run all tests"
    Write-Host "  .\run_all_tests.ps1 -TestType unit     # Run only unit tests"
    Write-Host "  .\run_all_tests.ps1 -Coverage          # Run with coverage"
    Write-Host "  .\run_all_tests.ps1 -Clean -Verbose    # Clean build with verbose output"
}

# Check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check Rust installation
    try {
        $rustVersion = cargo --version
        Write-Success "Rust found: $rustVersion"
    }
    catch {
        Write-Error "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
        exit 1
    }
    
    # Check if we're in the right directory
    if (!(Test-Path "Cargo.toml")) {
        Write-Error "Cargo.toml not found. Please run this script from the tests directory."
        exit 1
    }
    
    # Check for required test files
    $requiredDirs = @("integration", "api")
    foreach ($dir in $requiredDirs) {
        if (!(Test-Path $dir)) {
            Write-Warning "Directory '$dir' not found - some tests will be skipped"
        }
    }
    
    Write-Success "Prerequisites check completed"
}

# Setup test environment
function Initialize-TestEnvironment {
    Write-Info "Setting up test environment..."
    
    # Create temporary directories if needed
    $tempDirs = @("target\test-tmp", "target\coverage", "target\logs")
    foreach ($dir in $tempDirs) {
        if (!(Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    
    # Set verbose cargo output if requested
    if ($Verbose) {
        $env:CARGO_TERM_VERBOSE = "true"
    }
    
    Write-Success "Test environment ready"
}

# Cleanup function
function Invoke-Cleanup {
    Write-Info "Cleaning up test artifacts..."
    
    try {
        # Stop any running API servers
        Get-Process -Name "beacon-api" -ErrorAction SilentlyContinue | Stop-Process -Force
        
        # Clean temporary files
        $cleanupPaths = @(
            "target\test-tmp\*",
            "target\debug\deps\beacon_*",
            "*.log"
        )
        
        foreach ($path in $cleanupPaths) {
            if (Test-Path $path) {
                Remove-Item $path -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
        
        Write-Success "Cleanup completed"
    }
    catch {
        Write-Warning "Some cleanup operations failed: $_"
    }
}

# Run tests with timing
function Invoke-TestCategory {
    param(
        [string]$Category,
        [scriptblock]$Command
    )
    
    Write-Info "Running $Category tests..."
    $startTime = Get-Date
    
    try {
        $result = & $Command
        $endTime = Get-Date
        $duration = ($endTime - $startTime).TotalSeconds
        
        $testResult = @{
            Category = $Category
            Status = "PASSED"
            Duration = $duration
            Output = $result
        }
        
        $global:TestResults += $testResult
        Write-Success "$Category tests completed in $([math]::Round($duration, 2)) seconds"
        return $true
    }
    catch {
        $endTime = Get-Date
        $duration = ($endTime - $startTime).TotalSeconds
        
        $testResult = @{
            Category = $Category
            Status = "FAILED"
            Duration = $duration
            Error = $_.Exception.Message
        }
        
        $global:TestResults += $testResult
        Write-Error "$Category tests failed after $([math]::Round($duration, 2)) seconds: $_"
        return $false
    }
}

# Unit tests
function Invoke-UnitTests {
    Write-Info "Starting unit tests..."
    
    $unitTestCommand = {
        # Run standard unit tests
        cargo test --lib --bins --release
        
        # Run doctests
        cargo test --doc --release
    }
    
    return Invoke-TestCategory "Unit" $unitTestCommand
}

# Integration tests
function Invoke-IntegrationTests {
    Write-Info "Starting integration tests..."
    
    $integrationTestCommand = {
        # Run integration tests
        cargo test --test "*" --release
        
        # Run specific integration scenarios
        if (Test-Path "integration") {
            cargo test --release --manifest-path integration\Cargo.toml
        }
    }
    
    return Invoke-TestCategory "Integration" $integrationTestCommand
}

# API tests
function Invoke-APITests {
    if ($NoAPI) {
        Write-Warning "Skipping API tests (--no-api flag set)"
        return $true
    }
    
    Write-Info "Starting API tests..."
    
    $apiTestCommand = {
        # Start API server in background
        $apiJob = Start-Job -ScriptBlock {
            Set-Location $using:PWD
            $env:RUST_LOG = $using:env:RUST_LOG
            $env:DATABASE_URL = $using:env:DATABASE_URL
            $env:JWT_SECRET = $using:env:JWT_SECRET
            $env:API_PORT = $using:env:API_PORT
            
            cargo run --bin beacon-api --release
        }
        
        try {
            # Wait for API server to start
            Start-Sleep -Seconds 10
            
            # Check if API server is running
            $healthCheck = try {
                Invoke-WebRequest -Uri "http://localhost:$env:API_PORT/health" -TimeoutSec 5
            } catch {
                throw "API server health check failed: $_"
            }
            
            if ($healthCheck.StatusCode -ne 200) {
                throw "API server not responding correctly"
            }
            
            Write-Success "API server is running"
            
            # Run API tests
            if (Test-Path "api") {
                cargo test --release --manifest-path api\Cargo.toml
            }
            
            # Run PowerShell API tests if available
            if (Test-Path "..\test_api.ps1") {
                Write-Info "Running PowerShell API tests..."
                & "..\test_api.ps1"
            }
            
        }
        finally {
            # Stop API server
            Stop-Job $apiJob -ErrorAction SilentlyContinue
            Remove-Job $apiJob -ErrorAction SilentlyContinue
            
            # Force kill any remaining processes
            Get-Process -Name "beacon-api" -ErrorAction SilentlyContinue | Stop-Process -Force
        }
    }
    
    return Invoke-TestCategory "API" $apiTestCommand
}

# Performance tests
function Invoke-PerformanceTests {
    Write-Info "Starting performance tests..."
    
    $performanceTestCommand = {
        # Run benchmark tests
        cargo test --release --bench "*" --features bench
        
        # Run performance-specific tests
        cargo test --release --test "*performance*"
    }
    
    return Invoke-TestCategory "Performance" $performanceTestCommand
}

# Benchmark tests
function Invoke-BenchmarkTests {
    Write-Info "Starting benchmark tests..."
    
    $benchmarkCommand = {
        # Check if benchmarks are available
        if (Test-Path "benches") {
            cargo bench --bench "*"
        } else {
            Write-Warning "No benchmark directory found"
        }
    }
    
    return Invoke-TestCategory "Benchmarks" $benchmarkCommand
}

# Documentation tests
function Invoke-DocTests {
    Write-Info "Starting documentation tests..."
    
    $docTestCommand = {
        # Test documentation
        cargo test --doc --release
        
        # Check documentation generation
        cargo doc --no-deps --release
    }
    
    return Invoke-TestCategory "Documentation" $docTestCommand
}

# Run tests with coverage
function Invoke-WithCoverage {
    Write-Info "Running tests with code coverage..."
    
    # Check if cargo-tarpaulin is installed
    try {
        cargo tarpaulin --version | Out-Null
    }
    catch {
        Write-Warning "cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin
    }
    
    $coverageCommand = {
        # Run coverage analysis
        cargo tarpaulin --out Html --output-dir target\coverage --release
        
        Write-Info "Coverage report generated in target\coverage\tarpaulin-report.html"
    }
    
    return Invoke-TestCategory "Coverage" $coverageCommand
}

# Run specific test type
function Invoke-SpecificTest {
    param([string]$Type)
    
    $success = $true
    
    switch ($Type.ToLower()) {
        "unit" { $success = Invoke-UnitTests }
        "integration" { $success = Invoke-IntegrationTests }
        "api" { $success = Invoke-APITests }
        "performance" { $success = Invoke-PerformanceTests }
        "benchmarks" { $success = Invoke-BenchmarkTests }
        "doc" { $success = Invoke-DocTests }
        default { 
            Write-Error "Unknown test type: $Type"
            return $false
        }
    }
    
    return $success
}

# Run all tests
function Invoke-AllTests {
    Write-Info "Running complete test suite..."
    
    $allPassed = $true
    
    # Run each test category
    $testCategories = @(
        { Invoke-UnitTests },
        { Invoke-IntegrationTests },
        { Invoke-APITests },
        { Invoke-DocTests }
    )
    
    foreach ($testFunc in $testCategories) {
        $result = & $testFunc
        if (!$result) {
            $allPassed = $false
        }
    }
    
    # Run performance tests if specifically requested or in full suite
    if ($TestType -eq "all" -or $TestType -eq "performance") {
        $result = Invoke-PerformanceTests
        if (!$result) {
            $allPassed = $false
        }
    }
    
    return $allPassed
}

# Print test summary
function Show-TestSummary {
    $endTime = Get-Date
    $totalDuration = ($endTime - $global:StartTime).TotalSeconds
    
    Write-Host ""
    Write-Host "🧪 BEACON Test Suite Summary" -ForegroundColor Cyan
    Write-Host "=============================" -ForegroundColor Cyan
    Write-Host ""
    
    $passed = 0
    $failed = 0
    $totalTestDuration = 0
    
    foreach ($result in $global:TestResults) {
        $status = if ($result.Status -eq "PASSED") { "✅" } else { "❌" }
        $duration = [math]::Round($result.Duration, 2)
        
        Write-Host "$status $($result.Category): $($result.Status) ($duration s)" -ForegroundColor $(if ($result.Status -eq "PASSED") { "Green" } else { "Red" })
        
        if ($result.Status -eq "PASSED") {
            $passed++
        } else {
            $failed++
            if ($result.Error) {
                Write-Host "   Error: $($result.Error)" -ForegroundColor Red
            }
        }
        
        $totalTestDuration += $result.Duration
    }
    
    Write-Host ""
    Write-Host "Results:" -ForegroundColor Yellow
    Write-Host "  • Total Categories: $($global:TestResults.Count)"
    Write-Host "  • Passed: $passed"
    Write-Host "  • Failed: $failed"
    Write-Host "  • Test Duration: $([math]::Round($totalTestDuration, 2)) seconds"
    Write-Host "  • Total Duration: $([math]::Round($totalDuration, 2)) seconds"
    Write-Host ""
    
    if ($failed -eq 0) {
        Write-Success "🎉 All tests passed!"
        return $true
    } else {
        Write-Error "💥 $failed test categories failed!"
        return $false
    }
}

# Main execution function
function Invoke-Main {
    try {
        Write-Host "🧪 BEACON Blockchain Test Suite - Windows Edition" -ForegroundColor Cyan
        Write-Host "=================================================" -ForegroundColor Cyan
        Write-Host ""
        
        # Show help if requested
        if ($Help) {
            Show-Help
            return
        }
        
        # Check prerequisites
        Test-Prerequisites
        
        # Clean if requested
        if ($Clean) {
            Invoke-Cleanup
        }
        
        # Setup test environment
        Initialize-TestEnvironment
        
        # Run tests based on options
        $success = $false
        
        if ($Coverage) {
            $success = Invoke-WithCoverage
        } elseif ($TestType -ne "all") {
            $success = Invoke-SpecificTest $TestType
        } else {
            $success = Invoke-AllTests
        }
        
        # Show summary
        $finalResult = Show-TestSummary
        
        # Final cleanup if requested
        if ($Clean) {
            Invoke-Cleanup
        }
        
        # Exit with appropriate code
        if (!$finalResult) {
            exit 1
        }
        
    }
    catch {
        Write-Error "Test execution failed: $_"
        
        # Emergency cleanup
        try {
            Invoke-Cleanup
        }
        catch {
            Write-Warning "Emergency cleanup failed: $_"
        }
        
        exit 1
    }
}

# Trap cleanup on exit
trap {
    try {
        Invoke-Cleanup
    }
    catch {
        # Ignore cleanup errors on exit
    }
}

# Run main function
Invoke-Main
