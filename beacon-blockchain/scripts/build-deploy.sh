#!/bin/bash
# BEACON Blockchain - Build and Deployment Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DOCKER_REGISTRY="beacon-blockchain"
VERSION=${1:-"latest"}
BUILD_TYPE=${2:-"production"}

echo -e "${BLUE}🚀 BEACON Blockchain Build & Deployment Script${NC}"
echo -e "${BLUE}================================================${NC}"

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check Docker availability
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    if ! docker info &> /dev/null; then
        print_error "Docker daemon is not running"
        exit 1
    fi
    
    print_status "Docker is available"
}

# Clean up previous builds
cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up previous builds...${NC}"
    
    # Remove old containers
    docker container prune -f || true
    
    # Remove old images (optional)
    # docker image prune -f || true
    
    print_status "Cleanup completed"
}

# Build production images
build_production() {
    echo -e "${BLUE}🏗️  Building production images...${NC}"
    
    # Build main production image
    docker build \
        -t ${DOCKER_REGISTRY}/beacon-node:${VERSION} \
        -t ${DOCKER_REGISTRY}/beacon-node:latest \
        -f Dockerfile \
        .
    
    print_status "Production image built: ${DOCKER_REGISTRY}/beacon-node:${VERSION}"
}

# Build development images
build_development() {
    echo -e "${BLUE}🏗️  Building development images...${NC}"
    
    # Build development image
    docker build \
        -t ${DOCKER_REGISTRY}/beacon-node:${VERSION}-dev \
        -t ${DOCKER_REGISTRY}/beacon-node:dev \
        -f Dockerfile.dev \
        .
    
    print_status "Development image built: ${DOCKER_REGISTRY}/beacon-node:${VERSION}-dev"
}

# Deploy production stack
deploy_production() {
    echo -e "${BLUE}🚀 Deploying production stack...${NC}"
    
    # Create network if it doesn't exist
    docker network create beacon-network 2>/dev/null || true
    
    # Deploy production stack
    docker-compose -f docker-compose.yml up -d
    
    print_status "Production stack deployed"
    
    # Wait for services to be healthy
    echo -e "${YELLOW}⏳ Waiting for services to be healthy...${NC}"
    sleep 30
    
    # Check service health
    check_health
}

# Deploy development stack
deploy_development() {
    echo -e "${BLUE}🚀 Deploying development stack...${NC}"
    
    # Create network if it doesn't exist
    docker network create beacon-dev-network 2>/dev/null || true
    
    # Deploy development stack
    docker-compose -f docker-compose.dev.yml up -d
    
    print_status "Development stack deployed"
    
    # Wait for services to be healthy
    echo -e "${YELLOW}⏳ Waiting for services to be healthy...${NC}"
    sleep 30
    
    # Check service health
    check_health_dev
}

# Check health of production services
check_health() {
    echo -e "${BLUE}🔍 Checking service health...${NC}"
    
    # Check each node
    for i in {1..3}; do
        local port=$((3000 + i))
        if curl -f http://localhost:${port}/health &>/dev/null; then
            print_status "BEACON Node ${i} is healthy (port ${port})"
        else
            print_warning "BEACON Node ${i} is not responding (port ${port})"
        fi
    done
    
    # Check load balancer
    if curl -f http://localhost:80/health &>/dev/null; then
        print_status "Load balancer is healthy"
    else
        print_warning "Load balancer is not responding"
    fi
    
    # Check monitoring
    if curl -f http://localhost:9090 &>/dev/null; then
        print_status "Prometheus is healthy"
    else
        print_warning "Prometheus is not responding"
    fi
}

# Check health of development services
check_health_dev() {
    echo -e "${BLUE}🔍 Checking development service health...${NC}"
    
    # Check development node
    if curl -f http://localhost:3000/health &>/dev/null; then
        print_status "Development BEACON node is healthy"
    else
        print_warning "Development BEACON node is not responding"
    fi
    
    # Check development monitoring
    if curl -f http://localhost:9090 &>/dev/null; then
        print_status "Development Prometheus is healthy"
    else
        print_warning "Development Prometheus is not responding"
    fi
}

# Show deployment information
show_info() {
    echo -e "${BLUE}📋 Deployment Information${NC}"
    echo -e "${BLUE}=========================${NC}"
    
    if [ "$BUILD_TYPE" = "development" ]; then
        echo -e "${GREEN}Development Environment:${NC}"
        echo "  • API: http://localhost:3000"
        echo "  • Health: http://localhost:3000/health"
        echo "  • Monitoring: http://localhost:9090"
        echo "  • Database: localhost:5432"
        echo "  • Redis: localhost:6379"
    else
        echo -e "${GREEN}Production Environment:${NC}"
        echo "  • Load Balancer: http://localhost:80"
        echo "  • Node 1 API: http://localhost:3001"
        echo "  • Node 2 API: http://localhost:3002"
        echo "  • Node 3 API: http://localhost:3003"
        echo "  • Monitoring: http://localhost:9090"
        echo "  • Grafana: http://localhost:3000"
    fi
    
    echo ""
    echo -e "${YELLOW}Useful Commands:${NC}"
    echo "  • View logs: docker-compose logs -f"
    echo "  • Stop services: docker-compose down"
    echo "  • Restart: docker-compose restart"
    echo "  • Shell access: docker exec -it <container> /bin/bash"
}

# Main execution
main() {
    echo "Build Type: $BUILD_TYPE"
    echo "Version: $VERSION"
    echo ""
    
    check_docker
    cleanup
    
    if [ "$BUILD_TYPE" = "development" ]; then
        build_development
        deploy_development
    else
        build_production
        deploy_production
    fi
    
    show_info
    
    print_status "🎉 BEACON Blockchain deployment completed successfully!"
}

# Parse command line arguments
case "$BUILD_TYPE" in
    "production"|"prod")
        BUILD_TYPE="production"
        ;;
    "development"|"dev")
        BUILD_TYPE="development"
        ;;
    *)
        print_error "Invalid build type. Use 'production' or 'development'"
        exit 1
        ;;
esac

# Run main function
main
