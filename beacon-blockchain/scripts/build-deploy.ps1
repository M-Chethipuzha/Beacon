# BEACON Blockchain - Build and Deployment Script (PowerShell)
# Optimized for Windows environments

param(
    [string]$Version = "latest",
    [ValidateSet("production", "development", "prod", "dev")]
    [string]$BuildType = "production"
)

# Configuration
$DockerRegistry = "beacon-blockchain"
$ErrorActionPreference = "Stop"

# Colors for output
function Write-Success { param($Message) Write-Host "✅ $Message" -ForegroundColor Green }
function Write-Warning { param($Message) Write-Host "⚠️ $Message" -ForegroundColor Yellow }
function Write-Error { param($Message) Write-Host "❌ $Message" -ForegroundColor Red }
function Write-Info { param($Message) Write-Host "🔵 $Message" -ForegroundColor Blue }

Write-Info "🚀 BEACON Blockchain Build and Deployment Script"
Write-Info "================================================"

# Normalize build type
switch ($BuildType) {
    { $_ -in @("production", "prod") } { $BuildType = "production" }
    { $_ -in @("development", "dev") } { $BuildType = "development" }
}

Write-Host "Build Type: $BuildType"
Write-Host "Version: $Version"
Write-Host ""

# Check Docker availability
function Test-Docker {
    try {
        if (!(Get-Command docker -ErrorAction SilentlyContinue)) {
            Write-Error "Docker is not installed or not in PATH"
            exit 1
        }
        
        docker info | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Docker daemon is not running"
            exit 1
        }
        
        Write-Success "Docker is available"
    }
    catch {
        Write-Error "Docker check failed: $_"
        exit 1
    }
}

# Clean up previous builds
function Invoke-Cleanup {
    Write-Warning "🧹 Cleaning up previous builds..."
    
    try {
        # Remove old containers
        docker container prune -f | Out-Null
        
        Write-Success "Cleanup completed"
    }
    catch {
        Write-Warning "Cleanup encountered issues: $_"
    }
}

# Build production images
function Build-Production {
    Write-Info "🏗️ Building production images..."
    
    try {
        docker build `
            -t "$DockerRegistry/beacon-node:$Version" `
            -t "$DockerRegistry/beacon-node:latest" `
            -f Dockerfile `
            .
        
        if ($LASTEXITCODE -ne 0) {
            throw "Production build failed"
        }
        
        Write-Success "Production image built: $DockerRegistry/beacon-node:$Version"
    }
    catch {
        Write-Error "Production build failed: $_"
        exit 1
    }
}

# Build development images
function Build-Development {
    Write-Info "🏗️ Building development images..."
    
    try {
        docker build `
            -t "$DockerRegistry/beacon-node:$Version-dev" `
            -t "$DockerRegistry/beacon-node:dev" `
            -f Dockerfile.dev `
            .
        
        if ($LASTEXITCODE -ne 0) {
            throw "Development build failed"
        }
        
        Write-Success "Development image built: $DockerRegistry/beacon-node:$Version-dev"
    }
    catch {
        Write-Error "Development build failed: $_"
        exit 1
    }
}

# Deploy production stack
function Deploy-Production {
    Write-Info "🚀 Deploying production stack..."
    
    try {
        # Create network if it doesn't exist
        docker network create beacon-network 2>$null
        
        # Deploy production stack
        docker-compose -f docker-compose.yml up -d
        
        if ($LASTEXITCODE -ne 0) {
            throw "Production deployment failed"
        }
        
        Write-Success "Production stack deployed"
        
        # Wait for services to be healthy
        Write-Warning "⏳ Waiting for services to be healthy..."
        Start-Sleep -Seconds 30
        
        # Check service health
        Test-Health
    }
    catch {
        Write-Error "Production deployment failed: $_"
        exit 1
    }
}

# Deploy development stack
function Deploy-Development {
    Write-Info "🚀 Deploying development stack..."
    
    try {
        # Create network if it doesn't exist
        docker network create beacon-dev-network 2>$null
        
        # Deploy development stack
        docker-compose -f docker-compose.dev.yml up -d
        
        if ($LASTEXITCODE -ne 0) {
            throw "Development deployment failed"
        }
        
        Write-Success "Development stack deployed"
        
        # Wait for services to be healthy
        Write-Warning "⏳ Waiting for services to be healthy..."
        Start-Sleep -Seconds 30
        
        # Check service health
        Test-HealthDev
    }
    catch {
        Write-Error "Development deployment failed: $_"
        exit 1
    }
}

# Check health of production services
function Test-Health {
    Write-Info "🔍 Checking service health..."
    
    # Check each node
    for ($i = 1; $i -le 3; $i++) {
        $port = 3000 + $i
        try {
            $response = Invoke-WebRequest -Uri "http://localhost:$port/health" -TimeoutSec 5 -UseBasicParsing
            if ($response.StatusCode -eq 200) {
                Write-Success "BEACON Node $i is healthy (port $port)"
            }
        }
        catch {
            Write-Warning "BEACON Node $i is not responding (port $port)"
        }
    }
    
    # Check load balancer
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:80/health" -TimeoutSec 5 -UseBasicParsing
        if ($response.StatusCode -eq 200) {
            Write-Success "Load balancer is healthy"
        }
    }
    catch {
        Write-Warning "Load balancer is not responding"
    }
    
    # Check monitoring
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:9090" -TimeoutSec 5 -UseBasicParsing
        if ($response.StatusCode -eq 200) {
            Write-Success "Prometheus is healthy"
        }
    }
    catch {
        Write-Warning "Prometheus is not responding"
    }
}

# Check health of development services
function Test-HealthDev {
    Write-Info "🔍 Checking development service health..."
    
    # Check development node
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -TimeoutSec 5 -UseBasicParsing
        if ($response.StatusCode -eq 200) {
            Write-Success "Development BEACON node is healthy"
        }
    }
    catch {
        Write-Warning "Development BEACON node is not responding"
    }
    
    # Check development monitoring
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:9090" -TimeoutSec 5 -UseBasicParsing
        if ($response.StatusCode -eq 200) {
            Write-Success "Development Prometheus is healthy"
        }
    }
    catch {
        Write-Warning "Development Prometheus is not responding"
    }
}

# Show deployment information
function Show-Info {
    Write-Info "📋 Deployment Information"
    Write-Info "========================="
    
    if ($BuildType -eq "development") {
        Write-Success "Development Environment:"
        Write-Host "  • API: http://localhost:3000"
        Write-Host "  • Health: http://localhost:3000/health"
        Write-Host "  • Monitoring: http://localhost:9090"
        Write-Host "  • Database: localhost:5432"
        Write-Host "  • Redis: localhost:6379"
    }
    else {
        Write-Success "Production Environment:"
        Write-Host "  • Load Balancer: http://localhost:80"
        Write-Host "  • Node 1 API: http://localhost:3001"
        Write-Host "  • Node 2 API: http://localhost:3002"
        Write-Host "  • Node 3 API: http://localhost:3003"
        Write-Host "  • Monitoring: http://localhost:9090"
        Write-Host "  • Grafana: http://localhost:3000"
    }
    
    Write-Host ""
    Write-Warning "Useful Commands:"
    Write-Host "  • View logs: docker-compose logs -f"
    Write-Host "  • Stop services: docker-compose down"
    Write-Host "  • Restart: docker-compose restart"
    Write-Host "  • Shell access: docker exec -it <container> /bin/bash"
}

# Main execution
try {
    Test-Docker
    Invoke-Cleanup
    
    if ($BuildType -eq "development") {
        Build-Development
        Deploy-Development
    }
    else {
        Build-Production
        Deploy-Production
    }
    
    Show-Info
    
    Write-Success "🎉 BEACON Blockchain deployment completed successfully!"
}
catch {
    Write-Error "Deployment failed: $_"
    exit 1
}
