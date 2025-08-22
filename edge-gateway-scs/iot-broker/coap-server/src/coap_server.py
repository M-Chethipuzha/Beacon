"""
BEACON CoAP Server - Standalone CoAP Service
===========================================

Standalone CoAP server for IoT device communication in BEACON Edge Gateway.
"""

import asyncio
import logging
import yaml
import os
from typing import Dict, Any
from datetime import datetime

try:
    import aiocoap
    from aiocoap import Message, Code, Context
    from aiocoap.resource import Resource, Site
    COAP_AVAILABLE = True
except ImportError:
    COAP_AVAILABLE = False

logger = logging.getLogger(__name__)


class DeviceDataResource(Resource):
    """Resource for device data collection."""
    
    def __init__(self):
        super().__init__()
        self.device_data = {}
    
    async def render_get(self, request):
        """Handle GET requests for device data."""
        try:
            path_parts = request.opt.uri_path
            if len(path_parts) >= 2:
                device_id = path_parts[1]
                
                if device_id in self.device_data:
                    data = self.device_data[device_id]
                    payload = f"Device: {device_id}\nData: {data}\nTimestamp: {datetime.utcnow().isoformat()}"
                    return Message(payload=payload.encode(), content_format=0)
                else:
                    return Message(code=Code.NOT_FOUND, payload=b"Device not found")
            
            return Message(code=Code.BAD_REQUEST, payload=b"Invalid device ID")
            
        except Exception as e:
            logger.error(f"CoAP GET error: {e}")
            return Message(code=Code.INTERNAL_SERVER_ERROR)
    
    async def render_post(self, request):
        """Handle POST requests for device data submission."""
        try:
            path_parts = request.opt.uri_path
            if len(path_parts) >= 2:
                device_id = path_parts[1]
                
                # Store device data
                payload = request.payload.decode('utf-8') if request.payload else ""
                self.device_data[device_id] = {
                    'data': payload,
                    'timestamp': datetime.utcnow().isoformat(),
                    'content_format': request.opt.content_format
                }
                
                logger.info(f"Received data from device {device_id}: {payload[:100]}...")
                
                return Message(code=Code.CREATED, payload=b"Data received")
            
            return Message(code=Code.BAD_REQUEST, payload=b"Invalid device ID")
            
        except Exception as e:
            logger.error(f"CoAP POST error: {e}")
            return Message(code=Code.INTERNAL_SERVER_ERROR)


class WellKnownResource(Resource):
    """Well-known core resource for service discovery."""
    
    def __init__(self):
        super().__init__()
    
    async def render_get(self, request):
        """Return available resources."""
        resources = [
            '</device/{id}>;rt="device-data";ct=0',
            '</status>;rt="gateway-status";ct=0',
            '</config>;rt="gateway-config";ct=0'
        ]
        
        payload = ','.join(resources)
        return Message(payload=payload.encode(), content_format=40)  # application/link-format


class StatusResource(Resource):
    """Resource for gateway status information."""
    
    def __init__(self):
        super().__init__()
    
    async def render_get(self, request):
        """Return gateway status."""
        status = {
            'gateway': 'BEACON Edge Gateway CoAP Server',
            'status': 'online',
            'timestamp': datetime.utcnow().isoformat(),
            'version': '1.0.0'
        }
        
        payload = f"Status: {status['status']}\nTimestamp: {status['timestamp']}\nVersion: {status['version']}"
        return Message(payload=payload.encode(), content_format=0)


class ConfigResource(Resource):
    """Resource for gateway configuration."""
    
    def __init__(self, config: Dict[str, Any]):
        super().__init__()
        self.config = config
    
    async def render_get(self, request):
        """Return sanitized gateway configuration."""
        # Return only non-sensitive config information
        public_config = {
            'server_port': self.config.get('server_port', 5683),
            'max_message_size': self.config.get('max_message_size', 1024),
            'resources_available': ['device', 'status', 'config']
        }
        
        payload = '\n'.join([f"{k}: {v}" for k, v in public_config.items()])
        return Message(payload=payload.encode(), content_format=0)


class CoAPServer:
    """Standalone CoAP server for BEACON Edge Gateway."""
    
    def __init__(self, config_file: str = None):
        """Initialize CoAP server."""
        self.config = self._load_config(config_file)
        self.context = None
        self.running = False
        
        if not COAP_AVAILABLE:
            logger.error("aiocoap library not available. CoAP server cannot start.")
            raise ImportError("aiocoap library required for CoAP server")
    
    def _load_config(self, config_file: str) -> Dict[str, Any]:
        """Load configuration from file."""
        default_config = {
            'server_host': '0.0.0.0',
            'server_port': 5683,
            'max_message_size': 1024,
            'use_dtls': False,
            'log_level': 'INFO'
        }
        
        if config_file and os.path.exists(config_file):
            try:
                with open(config_file, 'r') as f:
                    file_config = yaml.safe_load(f)
                    default_config.update(file_config)
            except Exception as e:
                logger.warning(f"Failed to load config file {config_file}: {e}")
        
        return default_config
    
    async def start(self):
        """Start the CoAP server."""
        try:
            logger.info("Starting BEACON CoAP Server")
            
            # Create server context
            self.context = await Context.create_server_context(
                site=Site(),
                bind=(self.config['server_host'], self.config['server_port'])
            )
            
            # Add resources
            site = self.context.serversite
            
            # Well-known core resource
            site.add_resource(['.well-known', 'core'], WellKnownResource())
            
            # Device data resource
            site.add_resource(['device'], DeviceDataResource())
            
            # Status resource
            site.add_resource(['status'], StatusResource())
            
            # Configuration resource
            site.add_resource(['config'], ConfigResource(self.config))
            
            self.running = True
            
            logger.info(f"CoAP server started on {self.config['server_host']}:{self.config['server_port']}")
            logger.info("Available resources:")
            logger.info("  - /.well-known/core (service discovery)")
            logger.info("  - /device/{id} (device data)")
            logger.info("  - /status (gateway status)")
            logger.info("  - /config (gateway config)")
            
        except Exception as e:
            logger.error(f"Failed to start CoAP server: {e}")
            raise
    
    async def stop(self):
        """Stop the CoAP server."""
        try:
            if self.context:
                await self.context.shutdown()
                self.running = False
                logger.info("CoAP server stopped")
        except Exception as e:
            logger.error(f"Error stopping CoAP server: {e}")
    
    async def run_forever(self):
        """Run the server until stopped."""
        await self.start()
        
        try:
            # Keep the server running
            while self.running:
                await asyncio.sleep(1)
        except KeyboardInterrupt:
            logger.info("Received keyboard interrupt, shutting down")
        finally:
            await self.stop()


def setup_logging(level: str = "INFO"):
    """Setup logging configuration."""
    logging.basicConfig(
        level=getattr(logging, level.upper()),
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )


async def main():
    """Main entry point."""
    # Setup logging
    config_file = os.environ.get('COAP_CONFIG_FILE', 'config/coap.conf')
    
    # Create a simple YAML config if it doesn't exist
    if not os.path.exists(config_file):
        os.makedirs(os.path.dirname(config_file), exist_ok=True)
        default_config = {
            'server_host': '0.0.0.0',
            'server_port': 5683,
            'max_message_size': 1024,
            'log_level': 'INFO'
        }
        with open(config_file, 'w') as f:
            yaml.dump(default_config, f)
    
    # Load config to get log level
    try:
        with open(config_file, 'r') as f:
            config = yaml.safe_load(f)
            log_level = config.get('log_level', 'INFO')
    except Exception:
        log_level = 'INFO'
    
    setup_logging(log_level)
    
    # Start CoAP server
    server = CoAPServer(config_file)
    await server.run_forever()


if __name__ == "__main__":
    asyncio.run(main())
