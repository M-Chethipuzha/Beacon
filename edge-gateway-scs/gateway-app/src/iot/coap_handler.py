"""
BEACON Edge Gateway - CoAP Handler
=================================

This module implements Constrained Application Protocol (CoAP) support
for IoT device communication in resource-constrained environments.
"""

import asyncio
import logging
from typing import Dict, Any, Optional, Callable
from datetime import datetime
import json
import socket

try:
    import aiocoap
    from aiocoap import Message, Code, Context
    from aiocoap.resource import Resource, Site
    COAP_AVAILABLE = True
except ImportError:
    COAP_AVAILABLE = False

logger = logging.getLogger(__name__)


class DeviceResource(Resource):
    """CoAP resource for device communication."""
    
    def __init__(self, gateway_service):
        super().__init__()
        self.gateway_service = gateway_service
    
    async def render_get(self, request):
        """Handle GET requests for device status."""
        try:
            # Extract device ID from URI path
            path_parts = request.opt.uri_path
            if len(path_parts) >= 2 and path_parts[0] == 'device':
                device_id = path_parts[1]
                
                # Check policy for device access
                if hasattr(self.gateway_service, 'policy_enforcer'):
                    allowed = await self.gateway_service.policy_enforcer.check_access(
                        device_id, 'coap://device/status', 'read'
                    )
                    if not allowed:
                        return Message(code=Code.UNAUTHORIZED)
                
                # Return device status
                status = {
                    'device_id': device_id,
                    'timestamp': datetime.utcnow().isoformat(),
                    'status': 'online'
                }
                
                payload = json.dumps(status).encode('utf-8')
                return Message(payload=payload, content_format=0)  # text/plain
            
            return Message(code=Code.NOT_FOUND)
            
        except Exception as e:
            logger.error(f"CoAP GET error: {e}")
            return Message(code=Code.INTERNAL_SERVER_ERROR)
    
    async def render_post(self, request):
        """Handle POST requests for device data."""
        try:
            # Extract device ID from URI path
            path_parts = request.opt.uri_path
            if len(path_parts) >= 2 and path_parts[0] == 'device':
                device_id = path_parts[1]
                
                # Check policy for device access
                if hasattr(self.gateway_service, 'policy_enforcer'):
                    allowed = await self.gateway_service.policy_enforcer.check_access(
                        device_id, 'coap://device/data', 'write'
                    )
                    if not allowed:
                        return Message(code=Code.UNAUTHORIZED)
                
                # Process device data
                try:
                    payload = request.payload.decode('utf-8')
                    data = json.loads(payload) if payload else {}
                    
                    # Forward to MQTT if available
                    if hasattr(self.gateway_service, 'mqtt_handler'):
                        topic = f"beacon/{device_id}/device/data"
                        await self.gateway_service.mqtt_handler.publish_message(
                            topic, json.dumps(data)
                        )
                    
                    # Log to blockchain if available
                    if hasattr(self.gateway_service, 'blockchain_client'):
                        await self.gateway_service.blockchain_client.log_device_activity(
                            device_id, 'coap_data_received', data
                        )
                    
                    return Message(code=Code.CREATED)
                    
                except json.JSONDecodeError:
                    return Message(code=Code.BAD_REQUEST)
            
            return Message(code=Code.NOT_FOUND)
            
        except Exception as e:
            logger.error(f"CoAP POST error: {e}")
            return Message(code=Code.INTERNAL_SERVER_ERROR)


class PolicyResource(Resource):
    """CoAP resource for policy queries."""
    
    def __init__(self, gateway_service):
        super().__init__()
        self.gateway_service = gateway_service
    
    async def render_get(self, request):
        """Handle policy check requests."""
        try:
            # Extract query parameters
            query = request.opt.uri_query
            device_id = None
            resource = None
            action = None
            
            for param in query:
                if param.startswith('device='):
                    device_id = param.split('=', 1)[1]
                elif param.startswith('resource='):
                    resource = param.split('=', 1)[1]
                elif param.startswith('action='):
                    action = param.split('=', 1)[1]
            
            if not all([device_id, resource, action]):
                return Message(code=Code.BAD_REQUEST)
            
            # Check policy
            if hasattr(self.gateway_service, 'policy_enforcer'):
                allowed = await self.gateway_service.policy_enforcer.check_access(
                    device_id, resource, action
                )
                
                result = {'allowed': allowed, 'timestamp': datetime.utcnow().isoformat()}
                payload = json.dumps(result).encode('utf-8')
                return Message(payload=payload, content_format=0)
            
            return Message(code=Code.SERVICE_UNAVAILABLE)
            
        except Exception as e:
            logger.error(f"CoAP policy check error: {e}")
            return Message(code=Code.INTERNAL_SERVER_ERROR)


class GatewayResource(Resource):
    """CoAP resource for gateway information."""
    
    def __init__(self, gateway_service):
        super().__init__()
        self.gateway_service = gateway_service
    
    async def render_get(self, request):
        """Handle gateway status requests."""
        try:
            # Return gateway information
            info = {
                'gateway_id': self.gateway_service.config.get('gateway', {}).get('id', 'unknown'),
                'status': 'online',
                'timestamp': datetime.utcnow().isoformat(),
                'services': {
                    'mqtt': hasattr(self.gateway_service, 'mqtt_handler'),
                    'policy': hasattr(self.gateway_service, 'policy_enforcer'),
                    'blockchain': hasattr(self.gateway_service, 'blockchain_client')
                }
            }
            
            payload = json.dumps(info).encode('utf-8')
            return Message(payload=payload, content_format=0)
            
        except Exception as e:
            logger.error(f"CoAP gateway info error: {e}")
            return Message(code=Code.INTERNAL_SERVER_ERROR)


class CoAPHandler:
    """CoAP server for IoT device communication."""
    
    def __init__(self, gateway_service):
        """Initialize CoAP handler."""
        self.gateway_service = gateway_service
        self.context = None
        self.server_task = None
        self.host = "0.0.0.0"
        self.port = 5683
        
        if not COAP_AVAILABLE:
            logger.warning("aiocoap library not available. CoAP functionality disabled.")
    
    async def start(self):
        """Start CoAP server."""
        if not COAP_AVAILABLE:
            logger.warning("CoAP server not started - aiocoap library not available")
            return
        
        try:
            # Get configuration
            coap_config = self.gateway_service.config.get('coap', {})
            self.host = coap_config.get('server_host', '0.0.0.0')
            self.port = coap_config.get('server_port', 5683)
            
            # Create CoAP context
            self.context = await Context.create_server_context(
                site=Site(),
                bind=(self.host, self.port)
            )
            
            # Add resources
            site = self.context.serversite
            site.add_resource(['device'], DeviceResource(self.gateway_service))
            site.add_resource(['policy'], PolicyResource(self.gateway_service))
            site.add_resource(['gateway'], GatewayResource(self.gateway_service))
            
            logger.info(f"CoAP server started on {self.host}:{self.port}")
            
        except Exception as e:
            logger.error(f"Failed to start CoAP server: {e}")
            raise
    
    async def stop(self):
        """Stop CoAP server."""
        if not COAP_AVAILABLE:
            return
        
        try:
            if self.context:
                await self.context.shutdown()
            logger.info("CoAP server stopped")
        except Exception as e:
            logger.error(f"Error stopping CoAP server: {e}")
    
    async def get_status(self) -> Dict[str, Any]:
        """Get CoAP handler status."""
        return {
            'enabled': COAP_AVAILABLE,
            'running': self.context is not None,
            'host': self.host,
            'port': self.port,
            'timestamp': datetime.utcnow().isoformat()
        }
    
    async def send_message(self, host: str, port: int, uri_path: str, 
                          payload: str = "", method: str = "GET") -> Optional[str]:
        """Send CoAP message to device."""
        if not COAP_AVAILABLE:
            logger.warning("CoAP not available for sending messages")
            return None
        
        try:
            context = await Context.create_client_context()
            
            # Create message
            if method.upper() == "GET":
                code = Code.GET
            elif method.upper() == "POST":
                code = Code.POST
            elif method.upper() == "PUT":
                code = Code.PUT
            else:
                code = Code.GET
            
            message = Message(
                code=code,
                uri=f"coap://{host}:{port}/{uri_path}",
                payload=payload.encode('utf-8') if payload else b''
            )
            
            # Send message
            response = await context.request(message).response
            
            await context.shutdown()
            
            return response.payload.decode('utf-8') if response.payload else ""
            
        except Exception as e:
            logger.error(f"CoAP message send failed: {e}")
            return None
    
    async def discover_devices(self) -> list:
        """Discover CoAP devices on the network."""
        if not COAP_AVAILABLE:
            return []
        
        try:
            devices = []
            
            # Simple discovery by checking common CoAP ports
            # In production, use multicast discovery
            local_network = "192.168.1"  # Adjust for your network
            
            for i in range(1, 255):
                try:
                    host = f"{local_network}.{i}"
                    
                    # Try to connect with timeout
                    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
                    sock.settimeout(0.1)
                    
                    result = sock.connect_ex((host, self.port))
                    sock.close()
                    
                    if result == 0:
                        # Test CoAP connectivity
                        response = await self.send_message(host, self.port, ".well-known/core")
                        if response is not None:
                            devices.append({
                                'host': host,
                                'port': self.port,
                                'discovered_at': datetime.utcnow().isoformat()
                            })
                
                except Exception:
                    continue  # Skip failed connections
            
            logger.info(f"Discovered {len(devices)} CoAP devices")
            return devices
            
        except Exception as e:
            logger.error(f"CoAP device discovery failed: {e}")
            return []
