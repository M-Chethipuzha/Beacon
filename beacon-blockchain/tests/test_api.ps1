# BEACON API Test Script (PowerShell)
# This script demonstrates basic API functionality

Write-Host "🌟 ========================================" -ForegroundColor Yellow
Write-Host "🚀 BEACON API Test Script" -ForegroundColor Green
Write-Host "🌟 ========================================" -ForegroundColor Yellow
Write-Host ""

$ApiBase = "http://localhost:3000"

Write-Host "📋 Testing API endpoints..." -ForegroundColor Cyan
Write-Host ""

try {
    # Test health endpoint
    Write-Host "🔍 1. Health Check:" -ForegroundColor Yellow
    $healthResponse = Invoke-RestMethod -Uri "$ApiBase/health" -Method Get
    $healthResponse | ConvertTo-Json -Depth 3
    Write-Host ""

    # Test info endpoint
    Write-Host "ℹ️  2. Server Info:" -ForegroundColor Yellow
    $infoResponse = Invoke-RestMethod -Uri "$ApiBase/info" -Method Get
    $infoResponse | ConvertTo-Json -Depth 3
    Write-Host ""

    # Test login
    Write-Host "🔐 3. Authentication (Login):" -ForegroundColor Yellow
    $loginData = @{
        username = "admin"
        password = "admin123"
    } | ConvertTo-Json

    $tokenResponse = Invoke-RestMethod -Uri "$ApiBase/auth/login" -Method Post -Body $loginData -ContentType "application/json"
    $tokenResponse | ConvertTo-Json -Depth 3
    $token = $tokenResponse.access_token
    Write-Host ""

    # Test authenticated endpoints
    if ($token) {
        Write-Host "🔒 4. Authenticated Requests:" -ForegroundColor Yellow
        $headers = @{ Authorization = "Bearer $token" }
        
        Write-Host "   📊 Blockchain Info:" -ForegroundColor Cyan
        $blockchainInfo = Invoke-RestMethod -Uri "$ApiBase/api/v1/blockchain/info" -Method Get -Headers $headers
        $blockchainInfo | ConvertTo-Json -Depth 3
        Write-Host ""
        
        Write-Host "   📦 Latest Blocks:" -ForegroundColor Cyan
        $blocks = Invoke-RestMethod -Uri "$ApiBase/api/v1/blockchain/blocks?limit=3" -Method Get -Headers $headers
        $blocks | ConvertTo-Json -Depth 3
        Write-Host ""
        
        Write-Host "   💰 Transactions:" -ForegroundColor Cyan
        $transactions = Invoke-RestMethod -Uri "$ApiBase/api/v1/transactions?limit=3" -Method Get -Headers $headers
        $transactions | ConvertTo-Json -Depth 3
        Write-Host ""
        
        Write-Host "   🗃️  State Query:" -ForegroundColor Cyan
        $state = Invoke-RestMethod -Uri "$ApiBase/api/v1/state?key=test_key" -Method Get -Headers $headers
        $state | ConvertTo-Json -Depth 3
        Write-Host ""
    } else {
        Write-Host "❌ Authentication failed - skipping authenticated tests" -ForegroundColor Red
    }

} catch {
    Write-Host "❌ Error: $_" -ForegroundColor Red
    Write-Host "💡 Make sure the BEACON API server is running on port 3000" -ForegroundColor Yellow
}

Write-Host "🎉 Test completed!" -ForegroundColor Green
Write-Host ""
Write-Host "💡 To start the server:" -ForegroundColor Cyan
Write-Host "   cargo run --bin beacon-api" -ForegroundColor White
Write-Host ""
Write-Host "📖 Available endpoints:" -ForegroundColor Cyan
Write-Host "   GET  /health                     - Health check" -ForegroundColor White
Write-Host "   GET  /info                       - Server information" -ForegroundColor White
Write-Host "   POST /auth/login                 - Authentication" -ForegroundColor White
Write-Host "   GET  /api/v1/blockchain/info     - Blockchain information" -ForegroundColor White
Write-Host "   GET  /api/v1/blockchain/blocks   - Block listing" -ForegroundColor White
Write-Host "   GET  /api/v1/transactions        - Transaction listing" -ForegroundColor White
Write-Host "   GET  /api/v1/state               - State queries" -ForegroundColor White
Write-Host ""
