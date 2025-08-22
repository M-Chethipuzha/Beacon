"""
BEACON Edge Gateway - Device Registry Integration
===============================================

This module integrates with the chaincode-library device-registry
to manage device registration, authentication, and lifecycle.
"""

import asyncio
import logging
from typing import Dict, Any, Optional, List
from datetime import datetime
import json
import hashlib

logger = logging.getLogger(__name__)


class DeviceRegistry:
    """Device registry integration with chaincode library."""
    
    def __init__(self, blockchain_client):
        """Initialize device registry with blockchain client."""
        self.blockchain_client = blockchain_client
        self.local_devices = {}
        self.pending_registrations = {}
        
    async def register_device(self, device_id: str, device_data: Dict[str, Any]) -> bool:
        """
        Register a new device using chaincode library device-registry.
        
        Args:
            device_id: Unique device identifier
            device_data: Device information and metadata
            
        Returns:
            Registration success status
        """
        try:
            # Prepare device registration request
            registration_request = {
                'device_id': device_id,
                'device_data': device_data,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat(),
                'registration_hash': self._generate_registration_hash(device_id, device_data)
            }
            
            # Add to pending registrations
            self.pending_registrations[device_id] = registration_request
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'registerDevice',
                registration_request
            )
            
            success = result.get('success', False)
            if success:
                # Move from pending to registered
                if device_id in self.pending_registrations:
                    del self.pending_registrations[device_id]
                
                # Store device in local registry
                self.local_devices[device_id] = {
                    'device_data': device_data,
                    'registration_time': datetime.utcnow(),
                    'status': 'registered',
                    'blockchain_data': result.get('device_record', {})
                }
                
                logger.info(f"Device {device_id} registered successfully")
            else:
                logger.error(f"Device {device_id} registration failed: {result.get('error', 'unknown')}")
                # Keep in pending for retry
            
            return success
            
        except Exception as e:
            logger.error(f"Device registration failed: {e}")
            return False
    
    async def unregister_device(self, device_id: str, reason: str = "manual") -> bool:
        """
        Unregister a device from the registry.
        
        Args:
            device_id: Unique device identifier
            reason: Reason for unregistration
            
        Returns:
            Unregistration success status
        """
        try:
            unregistration_request = {
                'device_id': device_id,
                'reason': reason,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'unregisterDevice',
                unregistration_request
            )
            
            success = result.get('success', False)
            if success:
                # Remove from local registry
                if device_id in self.local_devices:
                    self.local_devices[device_id]['status'] = 'unregistered'
                    self.local_devices[device_id]['unregistration_time'] = datetime.utcnow()
                
                logger.info(f"Device {device_id} unregistered successfully")
            else:
                logger.error(f"Device {device_id} unregistration failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Device unregistration failed: {e}")
            return False
    
    async def authenticate_device(self, device_id: str, credentials: Dict[str, Any]) -> Dict[str, Any]:
        """
        Authenticate a device using chaincode library.
        
        Args:
            device_id: Unique device identifier
            credentials: Device authentication credentials
            
        Returns:
            Authentication result with token/status
        """
        try:
            auth_request = {
                'device_id': device_id,
                'credentials': credentials,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'authenticateDevice',
                auth_request
            )
            
            if result.get('authenticated', False):
                # Update local device status
                if device_id in self.local_devices:
                    self.local_devices[device_id]['last_auth'] = datetime.utcnow()
                    self.local_devices[device_id]['auth_token'] = result.get('auth_token')
                
                logger.info(f"Device {device_id} authenticated successfully")
            else:
                logger.warning(f"Device {device_id} authentication failed: {result.get('error', 'invalid_credentials')}")
            
            return result
            
        except Exception as e:
            logger.error(f"Device authentication failed: {e}")
            return {'authenticated': False, 'error': str(e)}
    
    async def update_device_info(self, device_id: str, updates: Dict[str, Any]) -> bool:
        """
        Update device information in the registry.
        
        Args:
            device_id: Unique device identifier
            updates: Updated device information
            
        Returns:
            Update success status
        """
        try:
            update_request = {
                'device_id': device_id,
                'updates': updates,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'updateDeviceInfo',
                update_request
            )
            
            success = result.get('success', False)
            if success:
                # Update local device data
                if device_id in self.local_devices:
                    self.local_devices[device_id]['device_data'].update(updates)
                    self.local_devices[device_id]['last_update'] = datetime.utcnow()
                
                logger.info(f"Device {device_id} information updated successfully")
            else:
                logger.error(f"Device {device_id} update failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Device info update failed: {e}")
            return False
    
    async def get_device_info(self, device_id: str) -> Optional[Dict[str, Any]]:
        """
        Get device information from the registry.
        
        Args:
            device_id: Unique device identifier
            
        Returns:
            Device information or None if not found
        """
        try:
            # Check local cache first
            if device_id in self.local_devices:
                local_data = self.local_devices[device_id]
                # Return cached data if recent
                if self._is_cache_fresh(local_data):
                    return local_data['device_data']
            
            # Query from blockchain
            query_request = {
                'device_id': device_id,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'getDeviceInfo',
                query_request
            )
            
            device_info = result.get('device_info')
            if device_info:
                # Update local cache
                self.local_devices[device_id] = {
                    'device_data': device_info,
                    'cache_time': datetime.utcnow(),
                    'status': 'cached'
                }
            
            return device_info
            
        except Exception as e:
            logger.error(f"Get device info failed: {e}")
            return None
    
    async def list_registered_devices(self, gateway_id: str = None) -> List[Dict[str, Any]]:
        """
        List all registered devices for this gateway or all gateways.
        
        Args:
            gateway_id: Optional gateway ID filter
            
        Returns:
            List of registered devices
        """
        try:
            query_request = {
                'gateway_id': gateway_id or await self._get_gateway_id()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'listRegisteredDevices',
                query_request
            )
            
            return result.get('devices', [])
            
        except Exception as e:
            logger.error(f"List registered devices failed: {e}")
            return []
    
    async def get_device_status(self, device_id: str) -> Dict[str, Any]:
        """
        Get device status from the registry.
        
        Args:
            device_id: Unique device identifier
            
        Returns:
            Device status information
        """
        try:
            query_request = {
                'device_id': device_id,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'getDeviceStatus',
                query_request
            )
            
            return result.get('status', {})
            
        except Exception as e:
            logger.error(f"Get device status failed: {e}")
            return {}
    
    async def update_device_status(self, device_id: str, status: str, metadata: Dict[str, Any] = None) -> bool:
        """
        Update device status in the registry.
        
        Args:
            device_id: Unique device identifier
            status: New device status
            metadata: Additional status metadata
            
        Returns:
            Update success status
        """
        try:
            status_request = {
                'device_id': device_id,
                'status': status,
                'metadata': metadata or {},
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'updateDeviceStatus',
                status_request
            )
            
            success = result.get('success', False)
            if success:
                # Update local device status
                if device_id in self.local_devices:
                    self.local_devices[device_id]['status'] = status
                    self.local_devices[device_id]['status_update_time'] = datetime.utcnow()
                
                logger.debug(f"Device {device_id} status updated to: {status}")
            
            return success
            
        except Exception as e:
            logger.error(f"Device status update failed: {e}")
            return False
    
    async def search_devices(self, criteria: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Search for devices based on criteria.
        
        Args:
            criteria: Search criteria (type, capabilities, location, etc.)
            
        Returns:
            List of matching devices
        """
        try:
            search_request = {
                'criteria': criteria,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'searchDevices',
                search_request
            )
            
            return result.get('devices', [])
            
        except Exception as e:
            logger.error(f"Device search failed: {e}")
            return []
    
    async def get_device_history(self, device_id: str, limit: int = 100) -> List[Dict[str, Any]]:
        """
        Get device history from the registry.
        
        Args:
            device_id: Unique device identifier
            limit: Maximum number of history records
            
        Returns:
            Device history records
        """
        try:
            query_request = {
                'device_id': device_id,
                'limit': limit,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library device-registry
            result = await self.blockchain_client.call_chaincode(
                'device-registry',
                'getDeviceHistory',
                query_request
            )
            
            return result.get('history', [])
            
        except Exception as e:
            logger.error(f"Get device history failed: {e}")
            return []
    
    def _generate_registration_hash(self, device_id: str, device_data: Dict[str, Any]) -> str:
        """Generate a hash for device registration verification."""
        hash_data = {
            'device_id': device_id,
            'device_type': device_data.get('device_type'),
            'mac_address': device_data.get('mac_address'),
            'timestamp': datetime.utcnow().isoformat()
        }
        hash_string = json.dumps(hash_data, sort_keys=True)
        return hashlib.sha256(hash_string.encode()).hexdigest()
    
    def _is_cache_fresh(self, local_data: Dict[str, Any], max_age: int = 300) -> bool:
        """Check if local cache data is still fresh (default 5 minutes)."""
        if 'cache_time' not in local_data:
            return False
        
        age = (datetime.utcnow() - local_data['cache_time']).total_seconds()
        return age < max_age
    
    async def _get_gateway_id(self) -> str:
        """Get the gateway ID from blockchain client."""
        return getattr(self.blockchain_client, 'gateway_id', 'unknown-gateway')
    
    def get_local_device_count(self) -> int:
        """Get count of locally cached devices."""
        return len(self.local_devices)
    
    def get_pending_registration_count(self) -> int:
        """Get count of pending device registrations."""
        return len(self.pending_registrations)
    
    async def get_registry_stats(self) -> Dict[str, Any]:
        """Get device registry statistics."""
        return {
            'local_devices': len(self.local_devices),
            'pending_registrations': len(self.pending_registrations),
            'registered_devices': len([d for d in self.local_devices.values() if d.get('status') == 'registered']),
            'total_registrations': getattr(self, '_total_registrations', 0),
            'total_authentications': getattr(self, '_total_authentications', 0),
            'failed_authentications': getattr(self, '_failed_authentications', 0)
        }
