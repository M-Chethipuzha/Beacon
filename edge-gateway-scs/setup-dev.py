#!/usr/bin/env python3
"""
BEACON Edge Gateway SCS - Development Setup Script
==================================================

Automated setup script for development environment.
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path
import platform
import shutil

def run_command(cmd, check=True, shell=False):
    """Run a command and handle errors."""
    print(f"Running: {cmd}")
    try:
        if isinstance(cmd, list):
            result = subprocess.run(cmd, check=check, capture_output=True, text=True, shell=shell)
        else:
            result = subprocess.run(cmd, check=check, capture_output=True, text=True, shell=True)
        
        if result.stdout:
            print(result.stdout)
        return result
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {e}")
        if e.stderr:
            print(f"Error output: {e.stderr}")
        if check:
            sys.exit(1)
        return e

def check_prerequisites():
    """Check if required tools are installed."""
    print("Checking prerequisites...")
    
    required_tools = {
        'python': ['python', '--version'],
        'docker': ['docker', '--version'],
        'docker-compose': ['docker-compose', '--version'],
        'git': ['git', '--version']
    }
    
    missing_tools = []
    
    for tool, cmd in required_tools.items():
        try:
            result = run_command(cmd, check=False)
            if result.returncode == 0:
                print(f"✓ {tool} is installed")
            else:
                missing_tools.append(tool)
        except FileNotFoundError:
            missing_tools.append(tool)
    
    if missing_tools:
        print(f"❌ Missing required tools: {', '.join(missing_tools)}")
        print("Please install the missing tools and run the setup again.")
        sys.exit(1)
    
    print("✓ All prerequisites are satisfied")

def setup_python_environment():
    """Set up Python virtual environment."""
    print("Setting up Python environment...")
    
    venv_path = Path('venv')
    
    if venv_path.exists():
        print("Virtual environment already exists")
        response = input("Do you want to recreate it? (y/N): ")
        if response.lower() == 'y':
            shutil.rmtree(venv_path)
        else:
            return
    
    # Create virtual environment
    run_command([sys.executable, '-m', 'venv', 'venv'])
    
    # Activate virtual environment and install dependencies
    if platform.system() == 'Windows':
        pip_cmd = 'venv\\Scripts\\pip.exe'
        python_cmd = 'venv\\Scripts\\python.exe'
    else:
        pip_cmd = 'venv/bin/pip'
        python_cmd = 'venv/bin/python'
    
    # Upgrade pip
    run_command([python_cmd, '-m', 'pip', 'install', '--upgrade', 'pip'])
    
    # Install requirements
    if Path('requirements.txt').exists():
        run_command([pip_cmd, 'install', '-r', 'requirements.txt'])
    
    # Install development requirements
    if Path('requirements-dev.txt').exists():
        run_command([pip_cmd, 'install', '-r', 'requirements-dev.txt'])
    
    print("✓ Python environment setup complete")

def setup_directories():
    """Create necessary directories."""
    print("Creating directory structure...")
    
    directories = [
        'gateway-app/data',
        'gateway-app/logs',
        'gateway-app/certs',
        'gateway-app/config/templates',
        'iot-broker/mosquitto/data',
        'iot-broker/mosquitto/logs',
        'vpn-client/openvpn/config',
        'vpn-client/wireguard/config',
        'monitoring/prometheus/data',
        'monitoring/grafana/data',
        'docs/examples',
        'tests/unit',
        'tests/integration',
        'scripts'
    ]
    
    for directory in directories:
        Path(directory).mkdir(parents=True, exist_ok=True)
        print(f"✓ Created {directory}")

def generate_certificates():
    """Generate development certificates."""
    print("Generating development certificates...")
    
    certs_dir = Path('gateway-app/certs')
    
    # Check if certificates already exist
    if (certs_dir / 'gateway.crt').exists():
        print("Certificates already exist")
        response = input("Do you want to regenerate them? (y/N): ")
        if response.lower() != 'y':
            return
    
    # Generate CA key and certificate
    run_command([
        'openssl', 'req', '-x509', '-newkey', 'rsa:4096',
        '-keyout', str(certs_dir / 'ca.key'),
        '-out', str(certs_dir / 'ca.crt'),
        '-days', '365', '-nodes',
        '-subj', '/C=US/ST=CA/L=San Francisco/O=BEACON/OU=Development/CN=BEACON Dev CA'
    ])
    
    # Generate gateway key and certificate
    run_command([
        'openssl', 'req', '-x509', '-newkey', 'rsa:4096',
        '-keyout', str(certs_dir / 'gateway.key'),
        '-out', str(certs_dir / 'gateway.crt'),
        '-days', '365', '-nodes',
        '-subj', '/C=US/ST=CA/L=San Francisco/O=BEACON/OU=Development/CN=localhost'
    ])
    
    # Set appropriate permissions
    if platform.system() != 'Windows':
        run_command(['chmod', '600', str(certs_dir / '*.key')], shell=True)
        run_command(['chmod', '644', str(certs_dir / '*.crt')], shell=True)
    
    print("✓ Certificates generated")

def setup_configuration():
    """Set up configuration files."""
    print("Setting up configuration files...")
    
    config_dir = Path('gateway-app/config')
    
    # Create default configuration if it doesn't exist
    if not (config_dir / 'gateway.toml').exists():
        default_config = '''[gateway]
id = "gateway-dev-001"
name = "BEACON Development Edge Gateway"
environment = "development"

[api]
host = "0.0.0.0"
port = 8081
cors_enabled = true

[database]
path = "data/gateway.db"

[mqtt]
broker_host = "localhost"
broker_port = 1883
username = "beacon-gateway"
password = "beacon_dev_password"

[coap]
host = "0.0.0.0"
port = 5683

[blockchain]
endpoint = "http://localhost:8080"

[logging]
level = "DEBUG"
file_enabled = true
file_path = "logs/gateway.log"
'''
        with open(config_dir / 'gateway.toml', 'w') as f:
            f.write(default_config)
        print("✓ Created default gateway configuration")
    
    # Create environment file
    if not (Path('.env')).exists():
        env_content = '''# Development Environment Variables
MQTT_USERNAME=beacon-gateway
MQTT_PASSWORD=beacon_dev_password
PROMETHEUS_USERNAME=beacon-metrics
PROMETHEUS_PASSWORD=beacon_dev_metrics
GRAFANA_ADMIN_PASSWORD=beacon_admin
DATABASE_ENCRYPTION_KEY=dev_encryption_key_32_characters
'''
        with open('.env', 'w') as f:
            f.write(env_content)
        print("✓ Created environment file")

def setup_docker():
    """Set up Docker environment."""
    print("Setting up Docker environment...")
    
    # Build Docker images
    if Path('Dockerfile').exists():
        run_command(['docker', 'build', '-t', 'beacon/edge-gateway:dev', '.'])
        print("✓ Built gateway Docker image")
    
    # Pull required images
    images = [
        'eclipse-mosquitto:2.0',
        'prom/prometheus:latest',
        'grafana/grafana:latest'
    ]
    
    for image in images:
        run_command(['docker', 'pull', image])
        print(f"✓ Pulled {image}")

def setup_git_hooks():
    """Set up Git pre-commit hooks."""
    print("Setting up Git hooks...")
    
    if not Path('.git').exists():
        print("Not a Git repository, skipping Git hooks setup")
        return
    
    # Install pre-commit if available
    try:
        run_command(['pre-commit', 'install'])
        print("✓ Pre-commit hooks installed")
    except:
        print("Pre-commit not available, skipping")

def run_tests():
    """Run basic tests to verify setup."""
    print("Running basic tests...")
    
    if platform.system() == 'Windows':
        python_cmd = 'venv\\Scripts\\python.exe'
    else:
        python_cmd = 'venv/bin/python'
    
    # Test Python imports
    test_imports = [
        'fastapi',
        'uvicorn',
        'paho.mqtt.client',
        'aiocoap',
        'prometheus_client'
    ]
    
    for module in test_imports:
        try:
            run_command([python_cmd, '-c', f'import {module}; print("✓ {module}")'])
        except:
            print(f"❌ Failed to import {module}")
    
    # Test Docker
    try:
        run_command(['docker', 'run', '--rm', 'hello-world'], check=False)
        print("✓ Docker is working")
    except:
        print("❌ Docker test failed")

def main():
    parser = argparse.ArgumentParser(description='BEACON Edge Gateway Development Setup')
    parser.add_argument('--skip-python', action='store_true', help='Skip Python environment setup')
    parser.add_argument('--skip-docker', action='store_true', help='Skip Docker setup')
    parser.add_argument('--skip-certs', action='store_true', help='Skip certificate generation')
    parser.add_argument('--skip-tests', action='store_true', help='Skip running tests')
    parser.add_argument('--quick', action='store_true', help='Quick setup (minimal components)')
    
    args = parser.parse_args()
    
    print("BEACON Edge Gateway SCS - Development Setup")
    print("=" * 50)
    
    # Check prerequisites
    check_prerequisites()
    
    # Create directories
    setup_directories()
    
    # Set up Python environment
    if not args.skip_python and not args.quick:
        setup_python_environment()
    
    # Generate certificates
    if not args.skip_certs:
        generate_certificates()
    
    # Set up configuration
    setup_configuration()
    
    # Set up Docker
    if not args.skip_docker and not args.quick:
        setup_docker()
    
    # Set up Git hooks
    if not args.quick:
        setup_git_hooks()
    
    # Run tests
    if not args.skip_tests and not args.quick:
        run_tests()
    
    print("\n" + "=" * 50)
    print("✓ Development setup complete!")
    print("\nNext steps:")
    print("1. Activate Python environment:")
    if platform.system() == 'Windows':
        print("   venv\\Scripts\\activate")
    else:
        print("   source venv/bin/activate")
    print("2. Start services:")
    print("   docker-compose up -d")
    print("3. Run the gateway:")
    print("   python gateway-app/main.py")
    print("4. Access the API:")
    print("   http://localhost:8081/docs")

if __name__ == '__main__':
    main()
