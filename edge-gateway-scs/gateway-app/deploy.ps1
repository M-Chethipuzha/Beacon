# BEACON Edge Gateway - PowerShell Deployment Script
# =================================================

param(
    [switch]$Build = $false,
    [switch]$Stop = $false,
    [switch]$Restart = $false,
    [switch]$Logs = $false
)

Write-Host "🚀 BEACON Edge Gateway - PowerShell Deployment" -ForegroundColor Green

# Function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Check Docker
if (-not (Test-Command "docker")) {
    Write-Host "❌ Docker is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Check docker-compose
if (-not (Test-Command "docker-compose")) {
    Write-Host "❌ docker-compose is not installed or not in PATH" -ForegroundColor Red
    exit 1
}

# Check if Docker is running
try {
    docker info | Out-Null
} catch {
    Write-Host "❌ Docker is not running. Please start Docker Desktop and try again." -ForegroundColor Red
    exit 1
}

# Handle stop request
if ($Stop) {
    Write-Host "🛑 Stopping BEACON Edge Gateway services..." -ForegroundColor Yellow
    docker-compose down
    Write-Host "✅ Services stopped" -ForegroundColor Green
    exit 0
}

# Handle logs request
if ($Logs) {
    Write-Host "📋 Showing BEACON Edge Gateway logs..." -ForegroundColor Blue
    docker-compose logs -f
    exit 0
}

# Handle restart request
if ($Restart) {
    Write-Host "🔄 Restarting BEACON Edge Gateway services..." -ForegroundColor Yellow
    docker-compose restart
    Write-Host "✅ Services restarted" -ForegroundColor Green
    exit 0
}

# Main deployment process
Write-Host "📁 Creating necessary directories..." -ForegroundColor Blue

$directories = @("data", "logs", "certs", "docker\mosquitto\data", "docker\mosquitto\log", "docker\monitoring\grafana\dashboards")

foreach ($dir in $directories) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Host "   Created: $dir" -ForegroundColor Gray
    }
}

# Generate certificates if they don't exist
if (-not (Test-Path "certs\gateway-cert.pem")) {
    Write-Host "🔐 Generating self-signed certificates..." -ForegroundColor Blue
    
    # Check if OpenSSL is available
    if (Test-Command "openssl") {
        # Generate CA private key
        & openssl genrsa -out "certs\ca-private.pem" 4096
        
        # Generate CA certificate
        & openssl req -new -x509 -key "certs\ca-private.pem" -out "certs\ca-cert.pem" -days 365 -subj "/C=US/ST=State/L=City/O=BEACON/OU=Edge Gateway/CN=BEACON-CA"
        
        # Generate gateway private key
        & openssl genrsa -out "certs\gateway-private.pem" 4096
        
        # Generate gateway certificate signing request
        & openssl req -new -key "certs\gateway-private.pem" -out "certs\gateway.csr" -subj "/C=US/ST=State/L=City/O=BEACON/OU=Edge Gateway/CN=beacon-gateway"
        
        # Generate gateway certificate
        & openssl x509 -req -in "certs\gateway.csr" -CA "certs\ca-cert.pem" -CAkey "certs\ca-private.pem" -CAcreateserial -out "certs\gateway-cert.pem" -days 365
        
        # Clean up CSR
        Remove-Item "certs\gateway.csr" -Force
        
        Write-Host "✅ Certificates generated successfully" -ForegroundColor Green
    } else {
        Write-Host "⚠️  OpenSSL not found. Using placeholder certificates." -ForegroundColor Yellow
        
        # Create placeholder certificates
        "-----BEGIN CERTIFICATE-----" | Out-File "certs\ca-cert.pem" -Encoding ASCII
        "PLACEHOLDER CA CERTIFICATE" | Out-File "certs\ca-cert.pem" -Append -Encoding ASCII
        "-----END CERTIFICATE-----" | Out-File "certs\ca-cert.pem" -Append -Encoding ASCII
        
        "-----BEGIN CERTIFICATE-----" | Out-File "certs\gateway-cert.pem" -Encoding ASCII
        "PLACEHOLDER GATEWAY CERTIFICATE" | Out-File "certs\gateway-cert.pem" -Append -Encoding ASCII
        "-----END CERTIFICATE-----" | Out-File "certs\gateway-cert.pem" -Append -Encoding ASCII
        
        "-----BEGIN PRIVATE KEY-----" | Out-File "certs\gateway-private.pem" -Encoding ASCII
        "PLACEHOLDER PRIVATE KEY" | Out-File "certs\gateway-private.pem" -Append -Encoding ASCII
        "-----END PRIVATE KEY-----" | Out-File "certs\gateway-private.pem" -Append -Encoding ASCII
    }
}

# Generate gateway salt for device ID hashing
if (-not (Test-Path "data\gateway-salt.txt")) {
    Write-Host "🧂 Generating gateway salt..." -ForegroundColor Blue
    
    # Generate random 32-byte salt
    $salt = -join ((1..64) | ForEach {'{0:X}' -f (Get-Random -Max 16)})
    $salt | Out-File "data\gateway-salt.txt" -Encoding ASCII -NoNewline
    
    Write-Host "✅ Gateway salt generated" -ForegroundColor Green
}

# Create MQTT password file
if (-not (Test-Path "docker\mosquitto\config\passwd")) {
    Write-Host "🔑 Creating MQTT password file..." -ForegroundColor Blue
    
    # Create password file with default gateway user
    "beacon-gateway:beacon_mqtt_password" | Out-File "docker\mosquitto\config\passwd" -Encoding ASCII
    
    Write-Host "✅ MQTT password file created" -ForegroundColor Green
}

# Build Docker images if requested
if ($Build) {
    Write-Host "🔨 Building Docker images..." -ForegroundColor Blue
    docker-compose build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Docker build failed" -ForegroundColor Red
        exit 1
    }
}

# Start services
Write-Host "🚀 Starting services..." -ForegroundColor Blue
docker-compose up -d

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to start services" -ForegroundColor Red
    exit 1
}

# Wait for services to be ready
Write-Host "⏳ Waiting for services to start..." -ForegroundColor Blue
Start-Sleep -Seconds 30

# Check service health
Write-Host "🏥 Checking service health..." -ForegroundColor Blue

# Function to check HTTP endpoint
function Test-HttpEndpoint($url, $name) {
    try {
        $response = Invoke-WebRequest -Uri $url -TimeoutSec 5 -UseBasicParsing
        if ($response.StatusCode -eq 200) {
            Write-Host "✅ $name is healthy" -ForegroundColor Green
            return $true
        }
    } catch {
        Write-Host "⚠️  $name is not responding" -ForegroundColor Yellow
        return $false
    }
}

# Function to check TCP port
function Test-TcpPort($hostname, $port, $name) {
    try {
        $tcpClient = New-Object System.Net.Sockets.TcpClient
        $tcpClient.ConnectAsync($hostname, $port).Wait(2000)
        if ($tcpClient.Connected) {
            Write-Host "✅ $name is running" -ForegroundColor Green
            $tcpClient.Close()
            return $true
        }
    } catch {
        Write-Host "⚠️  $name is not responding" -ForegroundColor Yellow
        return $false
    }
}

# Check services
Test-HttpEndpoint "http://localhost:8081/health" "Edge Gateway API"
Test-TcpPort "localhost" 1883 "MQTT broker"
Test-HttpEndpoint "http://localhost:9091" "Prometheus"
Test-HttpEndpoint "http://localhost:3000" "Grafana"

Write-Host ""
Write-Host "🎉 BEACON Edge Gateway deployment completed!" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Service URLs:" -ForegroundColor Cyan
Write-Host "   - Edge Gateway API: http://localhost:8081" -ForegroundColor White
Write-Host "   - Gateway Health:   http://localhost:8081/health" -ForegroundColor White
Write-Host "   - Prometheus:       http://localhost:9091" -ForegroundColor White
Write-Host "   - Grafana:          http://localhost:3000 (admin/beacon_admin)" -ForegroundColor White
Write-Host ""
Write-Host "📱 MQTT Connection:" -ForegroundColor Cyan
Write-Host "   - Host: localhost" -ForegroundColor White
Write-Host "   - Port: 1883 (plain) / 8883 (TLS)" -ForegroundColor White
Write-Host "   - WebSocket: 9001" -ForegroundColor White
Write-Host ""
Write-Host "📋 Management Commands:" -ForegroundColor Cyan
Write-Host "   - View logs:        .\deploy.ps1 -Logs" -ForegroundColor White
Write-Host "   - Restart services: .\deploy.ps1 -Restart" -ForegroundColor White
Write-Host "   - Stop services:    .\deploy.ps1 -Stop" -ForegroundColor White
Write-Host "   - Rebuild images:   .\deploy.ps1 -Build" -ForegroundColor White
