"""
BEACON Edge Gateway - Gateway Management Integration
==================================================

This module integrates with the chaincode-library gateway-management
to handle gateway registration, configuration, and lifecycle management.
"""

import asyncio
import logging
from typing import Dict, Any, Optional, List
from datetime import datetime
import json
import platform
import psutil

logger = logging.getLogger(__name__)


class GatewayManagement:
    """Gateway management integration with chaincode library."""
    
    def __init__(self, blockchain_client, config: Dict[str, Any]):
        """Initialize gateway management with blockchain client and config."""
        self.blockchain_client = blockchain_client
        self.config = config
        self.gateway_id = config.get('gateway', {}).get('id', 'unknown-gateway')
        self.registration_status = False
        self.last_heartbeat = None
        
    async def register_gateway(self) -> bool:
        """
        Register this gateway with the I&O SCS using chaincode library.
        
        Returns:
            Registration success status
        """
        try:
            # Collect gateway information
            gateway_info = await self._collect_gateway_info()
            
            registration_request = {
                'gateway_id': self.gateway_id,
                'gateway_info': gateway_info,
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library gateway-management
            result = await self.blockchain_client.call_chaincode(
                'gateway-management',
                'registerGateway',
                registration_request
            )
            
            success = result.get('success', False)
            if success:
                self.registration_status = True
                logger.info(f"Gateway {self.gateway_id} registered successfully")
                
                # Store registration details
                self.registration_data = result.get('registration_data', {})
            else:
                logger.error(f"Gateway registration failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Gateway registration failed: {e}")
            return False
    
    async def unregister_gateway(self, reason: str = "shutdown") -> bool:
        """
        Unregister this gateway from the I&O SCS.
        
        Args:
            reason: Reason for unregistration
            
        Returns:
            Unregistration success status
        """
        try:
            if not self.registration_status:
                logger.warning("Gateway not registered, skipping unregistration")
                return True
            
            unregistration_request = {
                'gateway_id': self.gateway_id,
                'reason': reason,
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library gateway-management
            result = await self.blockchain_client.call_chaincode(
                'gateway-management',
                'unregisterGateway',
                unregistration_request
            )
            
            success = result.get('success', False)
            if success:
                self.registration_status = False
                logger.info(f"Gateway {self.gateway_id} unregistered successfully")
            else:
                logger.error(f"Gateway unregistration failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Gateway unregistration failed: {e}")
            return False
    
    async def send_heartbeat(self) -> bool:
        """
        Send heartbeat to maintain gateway registration.
        
        Returns:
            Heartbeat success status
        """
        try:
            if not self.registration_status:
                logger.warning("Gateway not registered, cannot send heartbeat")
                return False
            
            # Collect current status
            status_info = await self._collect_status_info()
            
            heartbeat_request = {
                'gateway_id': self.gateway_id,
                'status': status_info,
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library gateway-management
            result = await self.blockchain_client.call_chaincode(
                'gateway-management',
                'gatewayHeartbeat',
                heartbeat_request
            )
            
            success = result.get('success', False)
            if success:
                self.last_heartbeat = datetime.utcnow()
                logger.debug(f"Heartbeat sent successfully for gateway {self.gateway_id}")
                
                # Process any configuration updates from the response
                config_updates = result.get('config_updates', {})
                if config_updates:
                    await self._apply_config_updates(config_updates)
            else:
                logger.warning(f"Heartbeat failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Heartbeat failed: {e}")
            return False
    
    async def update_gateway_config(self, config_updates: Dict[str, Any]) -> bool:
        """
        Update gateway configuration via chaincode library.
        
        Args:
            config_updates: Configuration updates to apply
            
        Returns:
            Update success status
        """
        try:
            update_request = {
                'gateway_id': self.gateway_id,
                'config_updates': config_updates,
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library gateway-management
            result = await self.blockchain_client.call_chaincode(
                'gateway-management',
                'updateGatewayConfig',
                update_request
            )
            
            success = result.get('success', False)
            if success:
                logger.info(f"Gateway configuration updated successfully")
                # Apply updates locally
                await self._apply_config_updates(config_updates)
            else:
                logger.error(f"Gateway config update failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Gateway config update failed: {e}")
            return False
    
    async def get_gateway_status(self) -> Dict[str, Any]:
        """
        Get gateway status from chaincode library.
        
        Returns:
            Gateway status information
        """
        try:
            query_request = {
                'gateway_id': self.gateway_id
            }
            
            # Call chaincode library gateway-management
            result = await self.blockchain_client.call_chaincode(
                'gateway-management',
                'getGatewayStatus',
                query_request
            )
            
            return result.get('status', {})
            
        except Exception as e:
            logger.error(f"Get gateway status failed: {e}")
            return {}
    
    async def list_connected_gateways(self) -> List[Dict[str, Any]]:
        """
        List all connected gateways in the network.
        
        Returns:
            List of connected gateways
        """
        try:
            # Call chaincode library gateway-management
            result = await self.blockchain_client.call_chaincode(
                'gateway-management',
                'listConnectedGateways',
                {}
            )
            
            return result.get('gateways', [])
            
        except Exception as e:
            logger.error(f"List connected gateways failed: {e}")
            return []
    
    async def get_gateway_metrics(self) -> Dict[str, Any]:
        """
        Get gateway performance metrics from chaincode library.
        
        Returns:
            Gateway metrics data
        """
        try:
            query_request = {
                'gateway_id': self.gateway_id
            }
            
            # Call chaincode library gateway-management
            result = await self.blockchain_client.call_chaincode(
                'gateway-management',
                'getGatewayMetrics',
                query_request
            )
            
            return result.get('metrics', {})
            
        except Exception as e:
            logger.error(f"Get gateway metrics failed: {e}")
            return {}
    
    async def report_incident(self, incident_type: str, description: str, 
                            severity: str = "medium", metadata: Dict[str, Any] = None) -> str:
        """
        Report an incident to the chaincode library.
        
        Args:
            incident_type: Type of incident
            description: Incident description
            severity: Incident severity (low, medium, high, critical)
            metadata: Additional incident metadata
            
        Returns:
            Incident ID or None if failed
        """
        try:
            incident_request = {
                'gateway_id': self.gateway_id,
                'incident_type': incident_type,
                'description': description,
                'severity': severity,
                'metadata': metadata or {},
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library gateway-management
            result = await self.blockchain_client.call_chaincode(
                'gateway-management',
                'reportIncident',
                incident_request
            )
            
            incident_id = result.get('incident_id')
            if incident_id:
                logger.info(f"Incident {incident_id} reported successfully")
            else:
                logger.error(f"Incident reporting failed: {result.get('error', 'unknown')}")
            
            return incident_id
            
        except Exception as e:
            logger.error(f"Incident reporting failed: {e}")
            return None
    
    async def _collect_gateway_info(self) -> Dict[str, Any]:
        """Collect comprehensive gateway information for registration."""
        try:
            return {
                'name': self.config.get('gateway', {}).get('name', 'BEACON Edge Gateway'),
                'location': self.config.get('gateway', {}).get('location', 'Unknown'),
                'capabilities': self.config.get('gateway', {}).get('capabilities', []),
                'version': '1.0.0',
                'platform': {
                    'system': platform.system(),
                    'release': platform.release(),
                    'architecture': platform.machine(),
                    'processor': platform.processor()
                },
                'hardware': {
                    'cpu_count': psutil.cpu_count(),
                    'memory_total': psutil.virtual_memory().total,
                    'disk_total': psutil.disk_usage('/').total if platform.system() != 'Windows' else psutil.disk_usage('C:').total
                },
                'network': {
                    'interfaces': self._get_network_interfaces()
                },
                'services': {
                    'mqtt_enabled': 'mqtt' in self.config,
                    'coap_enabled': 'coap' in self.config,
                    'api_enabled': 'api' in self.config,
                    'vpn_enabled': self.config.get('vpn', {}).get('enabled', False)
                },
                'configuration': {
                    'policy_cache_enabled': True,
                    'device_authentication': self.config.get('security', {}).get('enable_device_authentication', False),
                    'audit_logging': self.config.get('privacy', {}).get('audit_logging', False)
                }
            }
        except Exception as e:
            logger.error(f"Failed to collect gateway info: {e}")
            return {'error': str(e)}
    
    async def _collect_status_info(self) -> Dict[str, Any]:
        """Collect current gateway status information."""
        try:
            return {
                'status': 'online',
                'uptime': (datetime.utcnow() - getattr(self, 'start_time', datetime.utcnow())).total_seconds(),
                'cpu_usage': psutil.cpu_percent(interval=1),
                'memory_usage': psutil.virtual_memory().percent,
                'disk_usage': psutil.disk_usage('/').percent if platform.system() != 'Windows' else psutil.disk_usage('C:').percent,
                'connected_devices': getattr(self, 'connected_device_count', 0),
                'active_policies': getattr(self, 'active_policy_count', 0),
                'last_policy_sync': getattr(self, 'last_policy_sync', None),
                'services': {
                    'discovery': True,  # TODO: Get actual service status
                    'mqtt': True,
                    'coap': True,
                    'api': True
                }
            }
        except Exception as e:
            logger.error(f"Failed to collect status info: {e}")
            return {'status': 'error', 'error': str(e)}
    
    def _get_network_interfaces(self) -> List[Dict[str, str]]:
        """Get network interface information."""
        try:
            interfaces = []
            for interface, addresses in psutil.net_if_addrs().items():
                for address in addresses:
                    if address.family == 2:  # IPv4
                        interfaces.append({
                            'interface': interface,
                            'ip_address': address.address,
                            'netmask': address.netmask
                        })
            return interfaces
        except Exception as e:
            logger.error(f"Failed to get network interfaces: {e}")
            return []
    
    async def _apply_config_updates(self, config_updates: Dict[str, Any]):
        """Apply configuration updates received from chaincode."""
        try:
            for section, updates in config_updates.items():
                if section in self.config:
                    self.config[section].update(updates)
                    logger.info(f"Applied config updates to section: {section}")
                else:
                    logger.warning(f"Unknown config section in updates: {section}")
        except Exception as e:
            logger.error(f"Failed to apply config updates: {e}")
    
    async def start_heartbeat_timer(self, interval: int = 300):
        """Start periodic heartbeat timer (default 5 minutes)."""
        async def heartbeat_loop():
            while self.registration_status:
                try:
                    await self.send_heartbeat()
                    await asyncio.sleep(interval)
                except Exception as e:
                    logger.error(f"Heartbeat loop error: {e}")
                    await asyncio.sleep(60)  # Retry after 1 minute on error
        
        # Start heartbeat task
        asyncio.create_task(heartbeat_loop())
        logger.info(f"Started heartbeat timer with {interval}s interval")
    
    def is_registered(self) -> bool:
        """Check if gateway is currently registered."""
        return self.registration_status
    
    def get_last_heartbeat(self) -> Optional[datetime]:
        """Get timestamp of last successful heartbeat."""
        return self.last_heartbeat
