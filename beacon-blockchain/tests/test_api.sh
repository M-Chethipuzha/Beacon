#!/bin/bash

# BEACON API Test Script
# This script demonstrates basic API functionality

echo "🌟 ========================================"
echo "🚀 BEACON API Test Script"
echo "🌟 ========================================"
echo ""

API_BASE="http://localhost:3000"

echo "📋 Testing API endpoints..."
echo ""

# Test health endpoint
echo "🔍 1. Health Check:"
curl -s "$API_BASE/health" | jq '.'
echo ""

# Test info endpoint
echo "ℹ️  2. Server Info:"
curl -s "$API_BASE/info" | jq '.'
echo ""

# Test login
echo "🔐 3. Authentication (Login):"
TOKEN_RESPONSE=$(curl -s -X POST "$API_BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}')

echo "$TOKEN_RESPONSE" | jq '.'
TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.access_token')
echo ""

# Test authenticated endpoints
if [ "$TOKEN" != "null" ] && [ "$TOKEN" != "" ]; then
  echo "🔒 4. Authenticated Requests:"
  
  echo "   📊 Blockchain Info:"
  curl -s -H "Authorization: Bearer $TOKEN" "$API_BASE/api/v1/blockchain/info" | jq '.'
  echo ""
  
  echo "   📦 Latest Blocks:"
  curl -s -H "Authorization: Bearer $TOKEN" "$API_BASE/api/v1/blockchain/blocks?limit=3" | jq '.'
  echo ""
  
  echo "   💰 Transactions:"
  curl -s -H "Authorization: Bearer $TOKEN" "$API_BASE/api/v1/transactions?limit=3" | jq '.'
  echo ""
  
  echo "   🗃️  State Query:"
  curl -s -H "Authorization: Bearer $TOKEN" "$API_BASE/api/v1/state?key=test_key" | jq '.'
  echo ""
  
else
  echo "❌ Authentication failed - skipping authenticated tests"
fi

echo "🎉 Test completed!"
echo ""
echo "💡 To start the server:"
echo "   cargo run --bin beacon-api"
echo ""
echo "📖 Available endpoints:"
echo "   GET  /health                     - Health check"
echo "   GET  /info                       - Server information"
echo "   POST /auth/login                 - Authentication"
echo "   GET  /api/v1/blockchain/info     - Blockchain information"
echo "   GET  /api/v1/blockchain/blocks   - Block listing"
echo "   GET  /api/v1/transactions        - Transaction listing"
echo "   GET  /api/v1/state               - State queries"
echo ""
