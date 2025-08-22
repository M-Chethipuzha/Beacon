"""
BEACON Edge Gateway - Local API Server
=====================================

This module implements a local REST API server for the Edge Gateway,
providing endpoints for device management, policy queries, and gateway
status monitoring.
"""

from aiohttp import web, cors
import aiohttp_cors
import asyncio
import json
import logging
from typing import Dict, Any, Optional
from datetime import datetime

logger = logging.getLogger(__name__)


class APIServer:
    """Local REST API server for Edge Gateway management."""
    
    def __init__(self, gateway_service):
        """Initialize API server with gateway service reference."""
        self.gateway_service = gateway_service
        self.app = None
        self.runner = None
        self.site = None
        
    async def start(self, host: str = "0.0.0.0", port: int = 8081):
        """Start the API server."""
        try:
            self.app = web.Application()
            
            # Configure CORS
            cors_config = aiohttp_cors.setup(self.app, defaults={
                "*": aiohttp_cors.ResourceOptions(
                    allow_credentials=True,
                    expose_headers="*",
                    allow_headers="*",
                    allow_methods="*"
                )
            })
            
            # Add routes
            self._setup_routes()
            
            # Add CORS to all routes
            for route in list(self.app.router.routes()):
                cors_config.add(route)
            
            # Start server
            self.runner = web.AppRunner(self.app)
            await self.runner.setup()
            
            self.site = web.TCPSite(self.runner, host, port)
            await self.site.start()
            
            logger.info(f"API server started on http://{host}:{port}")
            
        except Exception as e:
            logger.error(f"Failed to start API server: {e}")
            raise
    
    async def stop(self):
        """Stop the API server."""
        try:
            if self.site:
                await self.site.stop()
            if self.runner:
                await self.runner.cleanup()
            logger.info("API server stopped")
        except Exception as e:
            logger.error(f"Error stopping API server: {e}")
    
    def _setup_routes(self):
        """Setup API routes."""
        # Health and status endpoints
        self.app.router.add_get('/health', self.health_check)
        self.app.router.add_get('/status', self.get_status)
        self.app.router.add_get('/metrics', self.get_metrics)
        
        # Device management endpoints
        self.app.router.add_get('/devices', self.list_devices)
        self.app.router.add_get('/devices/{device_id}', self.get_device)
        self.app.router.add_post('/devices/{device_id}/authorize', self.authorize_device)
        self.app.router.add_delete('/devices/{device_id}', self.remove_device)
        
        # Policy management endpoints
        self.app.router.add_get('/policies', self.list_policies)
        self.app.router.add_get('/policies/{policy_id}', self.get_policy)
        self.app.router.add_post('/policies/check', self.check_policy)
        self.app.router.add_post('/policies/sync', self.sync_policies)
        
        # IoT communication endpoints
        self.app.router.add_post('/mqtt/publish', self.mqtt_publish)
        self.app.router.add_get('/mqtt/subscriptions', self.mqtt_subscriptions)
        
        # Configuration endpoints
        self.app.router.add_get('/config', self.get_config)
        self.app.router.add_put('/config', self.update_config)
    
    async def health_check(self, request):
        """Health check endpoint."""
        try:
            # Check if all services are running
            services_status = {
                'discovery': hasattr(self.gateway_service, 'discovery') and self.gateway_service.discovery is not None,
                'policy': hasattr(self.gateway_service, 'policy_enforcer') and self.gateway_service.policy_enforcer is not None,
                'mqtt': hasattr(self.gateway_service, 'mqtt_handler') and self.gateway_service.mqtt_handler is not None,
                'blockchain': hasattr(self.gateway_service, 'blockchain_client') and self.gateway_service.blockchain_client is not None
            }
            
            all_healthy = all(services_status.values())
            
            response = {
                'status': 'healthy' if all_healthy else 'degraded',
                'timestamp': datetime.utcnow().isoformat(),
                'services': services_status
            }
            
            status_code = 200 if all_healthy else 503
            return web.json_response(response, status=status_code)
            
        except Exception as e:
            logger.error(f"Health check failed: {e}")
            return web.json_response({
                'status': 'unhealthy',
                'error': str(e),
                'timestamp': datetime.utcnow().isoformat()
            }, status=500)
    
    async def get_status(self, request):
        """Get detailed gateway status."""
        try:
            # Collect status from all components
            status = {
                'gateway_id': self.gateway_service.config.get('gateway', {}).get('id', 'unknown'),
                'timestamp': datetime.utcnow().isoformat(),
                'uptime': getattr(self.gateway_service, 'start_time', datetime.utcnow()).isoformat(),
                'services': {}
            }
            
            # Discovery service status
            if hasattr(self.gateway_service, 'discovery') and self.gateway_service.discovery:
                discovery_status = await self.gateway_service.discovery.get_status()
                status['services']['discovery'] = discovery_status
            
            # Policy enforcer status
            if hasattr(self.gateway_service, 'policy_enforcer') and self.gateway_service.policy_enforcer:
                policy_status = await self.gateway_service.policy_enforcer.get_status()
                status['services']['policy'] = policy_status
            
            # MQTT handler status
            if hasattr(self.gateway_service, 'mqtt_handler') and self.gateway_service.mqtt_handler:
                mqtt_status = await self.gateway_service.mqtt_handler.get_status()
                status['services']['mqtt'] = mqtt_status
            
            return web.json_response(status)
            
        except Exception as e:
            logger.error(f"Status check failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def get_metrics(self, request):
        """Get Prometheus-style metrics."""
        try:
            metrics = []
            
            # Basic gateway metrics
            metrics.append('# HELP beacon_gateway_info Gateway information')
            metrics.append('# TYPE beacon_gateway_info gauge')
            metrics.append(f'beacon_gateway_info{{version="1.0.0"}} 1')
            
            # Service status metrics
            if hasattr(self.gateway_service, 'discovery') and self.gateway_service.discovery:
                status = await self.gateway_service.discovery.get_status()
                healthy = 1 if status.get('healthy', False) else 0
                metrics.append(f'beacon_service_health{{service="discovery"}} {healthy}')
            
            if hasattr(self.gateway_service, 'policy_enforcer') and self.gateway_service.policy_enforcer:
                status = await self.gateway_service.policy_enforcer.get_status()
                healthy = 1 if status.get('healthy', False) else 0
                metrics.append(f'beacon_service_health{{service="policy"}} {healthy}')
            
            response_text = '\n'.join(metrics)
            return web.Response(text=response_text, content_type='text/plain')
            
        except Exception as e:
            logger.error(f"Metrics generation failed: {e}")
            return web.Response(text='', status=500)
    
    async def list_devices(self, request):
        """List registered devices."""
        try:
            if not hasattr(self.gateway_service, 'mqtt_handler') or not self.gateway_service.mqtt_handler:
                return web.json_response({'error': 'MQTT handler not available'}, status=503)
            
            devices = await self.gateway_service.mqtt_handler.list_devices()
            return web.json_response({'devices': devices})
            
        except Exception as e:
            logger.error(f"List devices failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def get_device(self, request):
        """Get device information."""
        device_id = request.match_info['device_id']
        try:
            if not hasattr(self.gateway_service, 'mqtt_handler') or not self.gateway_service.mqtt_handler:
                return web.json_response({'error': 'MQTT handler not available'}, status=503)
            
            device = await self.gateway_service.mqtt_handler.get_device(device_id)
            if device:
                return web.json_response(device)
            else:
                return web.json_response({'error': 'Device not found'}, status=404)
                
        except Exception as e:
            logger.error(f"Get device failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def authorize_device(self, request):
        """Authorize a device for access."""
        device_id = request.match_info['device_id']
        try:
            data = await request.json()
            
            if not hasattr(self.gateway_service, 'policy_enforcer') or not self.gateway_service.policy_enforcer:
                return web.json_response({'error': 'Policy enforcer not available'}, status=503)
            
            # Add device authorization
            result = await self.gateway_service.policy_enforcer.authorize_device(
                device_id, 
                data.get('device_type', 'unknown'),
                data.get('capabilities', [])
            )
            
            return web.json_response({'success': result})
            
        except Exception as e:
            logger.error(f"Device authorization failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def remove_device(self, request):
        """Remove device from gateway."""
        device_id = request.match_info['device_id']
        try:
            if not hasattr(self.gateway_service, 'mqtt_handler') or not self.gateway_service.mqtt_handler:
                return web.json_response({'error': 'MQTT handler not available'}, status=503)
            
            result = await self.gateway_service.mqtt_handler.remove_device(device_id)
            return web.json_response({'success': result})
            
        except Exception as e:
            logger.error(f"Remove device failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def list_policies(self, request):
        """List cached policies."""
        try:
            if not hasattr(self.gateway_service, 'policy_enforcer') or not self.gateway_service.policy_enforcer:
                return web.json_response({'error': 'Policy enforcer not available'}, status=503)
            
            policies = await self.gateway_service.policy_enforcer.list_policies()
            return web.json_response({'policies': policies})
            
        except Exception as e:
            logger.error(f"List policies failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def get_policy(self, request):
        """Get specific policy."""
        policy_id = request.match_info['policy_id']
        try:
            if not hasattr(self.gateway_service, 'policy_enforcer') or not self.gateway_service.policy_enforcer:
                return web.json_response({'error': 'Policy enforcer not available'}, status=503)
            
            policy = await self.gateway_service.policy_enforcer.get_policy(policy_id)
            if policy:
                return web.json_response(policy)
            else:
                return web.json_response({'error': 'Policy not found'}, status=404)
                
        except Exception as e:
            logger.error(f"Get policy failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def check_policy(self, request):
        """Check policy for a specific request."""
        try:
            data = await request.json()
            
            if not hasattr(self.gateway_service, 'policy_enforcer') or not self.gateway_service.policy_enforcer:
                return web.json_response({'error': 'Policy enforcer not available'}, status=503)
            
            result = await self.gateway_service.policy_enforcer.check_access(
                data.get('device_id'),
                data.get('resource'),
                data.get('action'),
                data.get('context', {})
            )
            
            return web.json_response({'decision': result})
            
        except Exception as e:
            logger.error(f"Policy check failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def sync_policies(self, request):
        """Trigger policy synchronization with I&O SCS."""
        try:
            if not hasattr(self.gateway_service, 'policy_enforcer') or not self.gateway_service.policy_enforcer:
                return web.json_response({'error': 'Policy enforcer not available'}, status=503)
            
            result = await self.gateway_service.policy_enforcer.sync_policies()
            return web.json_response({'success': result})
            
        except Exception as e:
            logger.error(f"Policy sync failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def mqtt_publish(self, request):
        """Publish message via MQTT."""
        try:
            data = await request.json()
            
            if not hasattr(self.gateway_service, 'mqtt_handler') or not self.gateway_service.mqtt_handler:
                return web.json_response({'error': 'MQTT handler not available'}, status=503)
            
            result = await self.gateway_service.mqtt_handler.publish_message(
                data.get('topic'),
                data.get('payload', ''),
                data.get('qos', 1)
            )
            
            return web.json_response({'success': result})
            
        except Exception as e:
            logger.error(f"MQTT publish failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def mqtt_subscriptions(self, request):
        """Get active MQTT subscriptions."""
        try:
            if not hasattr(self.gateway_service, 'mqtt_handler') or not self.gateway_service.mqtt_handler:
                return web.json_response({'error': 'MQTT handler not available'}, status=503)
            
            subscriptions = await self.gateway_service.mqtt_handler.get_subscriptions()
            return web.json_response({'subscriptions': subscriptions})
            
        except Exception as e:
            logger.error(f"Get MQTT subscriptions failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def get_config(self, request):
        """Get gateway configuration (sanitized)."""
        try:
            # Return sanitized config (remove sensitive data)
            sanitized_config = self.gateway_service.config.copy()
            
            # Remove sensitive sections
            if 'mqtt' in sanitized_config:
                sanitized_config['mqtt'].pop('password', None)
            if 'vpn' in sanitized_config:
                sanitized_config['vpn'] = {'enabled': sanitized_config['vpn'].get('enabled', False)}
            
            return web.json_response({'config': sanitized_config})
            
        except Exception as e:
            logger.error(f"Get config failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
    
    async def update_config(self, request):
        """Update gateway configuration."""
        try:
            data = await request.json()
            
            # Validate and update config
            # For security, only allow certain fields to be updated
            allowed_updates = ['logging', 'monitoring', 'policy']
            
            for section in allowed_updates:
                if section in data:
                    if section not in self.gateway_service.config:
                        self.gateway_service.config[section] = {}
                    self.gateway_service.config[section].update(data[section])
            
            return web.json_response({'success': True, 'message': 'Configuration updated'})
            
        except Exception as e:
            logger.error(f"Update config failed: {e}")
            return web.json_response({'error': str(e)}, status=500)
