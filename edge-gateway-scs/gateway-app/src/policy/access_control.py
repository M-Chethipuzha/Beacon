"""
BEACON Edge Gateway - Access Control Integration
==============================================

This module integrates with the chaincode-library access-control
functionality to provide real-time access control decisions.
"""

import asyncio
import logging
from typing import Dict, Any, Optional, List
from datetime import datetime
import json

logger = logging.getLogger(__name__)


class AccessControl:
    """Access control integration with chaincode library."""
    
    def __init__(self, blockchain_client):
        """Initialize access control with blockchain client."""
        self.blockchain_client = blockchain_client
        self.cache = {}
        self.cache_ttl = 300  # 5 minutes cache TTL
        
    async def check_device_access(self, device_id: str, resource: str, 
                                action: str, context: Dict[str, Any] = None) -> Dict[str, Any]:
        """
        Check device access using chaincode library access-control.
        
        Args:
            device_id: Unique device identifier
            resource: Resource being accessed
            action: Action being performed (read, write, execute)
            context: Additional context information
            
        Returns:
            Access decision with metadata
        """
        try:
            # Prepare request for chaincode
            access_request = {
                'device_id': device_id,
                'resource': resource,
                'action': action,
                'context': context or {},
                'timestamp': datetime.utcnow().isoformat(),
                'gateway_id': await self._get_gateway_id()
            }
            
            # Check cache first
            cache_key = self._generate_cache_key(device_id, resource, action)
            if self._is_cache_valid(cache_key):
                logger.debug(f"Access control cache hit for {cache_key}")
                return self.cache[cache_key]['result']
            
            # Call chaincode library access-control
            result = await self.blockchain_client.call_chaincode(
                'access-control',
                'checkAccess',
                access_request
            )
            
            # Cache the result
            self.cache[cache_key] = {
                'result': result,
                'timestamp': datetime.utcnow(),
                'ttl': self.cache_ttl
            }
            
            logger.info(f"Access control decision for {device_id}: {result.get('decision', 'unknown')}")
            return result
            
        except Exception as e:
            logger.error(f"Access control check failed: {e}")
            # Return safe default
            return {
                'decision': 'deny',
                'reason': 'access_control_error',
                'timestamp': datetime.utcnow().isoformat(),
                'error': str(e)
            }
    
    async def register_device_access(self, device_id: str, device_type: str, 
                                   capabilities: List[str], metadata: Dict[str, Any] = None) -> bool:
        """
        Register device for access control using chaincode library.
        
        Args:
            device_id: Unique device identifier
            device_type: Type of device (sensor, actuator, etc.)
            capabilities: List of device capabilities
            metadata: Additional device metadata
            
        Returns:
            Registration success status
        """
        try:
            registration_request = {
                'device_id': device_id,
                'device_type': device_type,
                'capabilities': capabilities,
                'metadata': metadata or {},
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library access-control
            result = await self.blockchain_client.call_chaincode(
                'access-control',
                'registerDevice',
                registration_request
            )
            
            success = result.get('success', False)
            if success:
                logger.info(f"Device {device_id} registered successfully for access control")
                # Clear cache for this device
                self._clear_device_cache(device_id)
            else:
                logger.warning(f"Device {device_id} registration failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Device access registration failed: {e}")
            return False
    
    async def revoke_device_access(self, device_id: str, reason: str = None) -> bool:
        """
        Revoke device access using chaincode library.
        
        Args:
            device_id: Unique device identifier
            reason: Reason for revocation
            
        Returns:
            Revocation success status
        """
        try:
            revocation_request = {
                'device_id': device_id,
                'reason': reason or 'manual_revocation',
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library access-control
            result = await self.blockchain_client.call_chaincode(
                'access-control',
                'revokeDevice',
                revocation_request
            )
            
            success = result.get('success', False)
            if success:
                logger.info(f"Device {device_id} access revoked successfully")
                # Clear cache for this device
                self._clear_device_cache(device_id)
            else:
                logger.warning(f"Device {device_id} revocation failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Device access revocation failed: {e}")
            return False
    
    async def get_device_permissions(self, device_id: str) -> Dict[str, Any]:
        """
        Get device permissions from chaincode library.
        
        Args:
            device_id: Unique device identifier
            
        Returns:
            Device permissions and metadata
        """
        try:
            query_request = {
                'device_id': device_id,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library access-control
            result = await self.blockchain_client.call_chaincode(
                'access-control',
                'getDevicePermissions',
                query_request
            )
            
            return result
            
        except Exception as e:
            logger.error(f"Get device permissions failed: {e}")
            return {'permissions': [], 'error': str(e)}
    
    async def list_authorized_devices(self) -> List[Dict[str, Any]]:
        """
        List all authorized devices for this gateway.
        
        Returns:
            List of authorized devices
        """
        try:
            query_request = {
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library access-control
            result = await self.blockchain_client.call_chaincode(
                'access-control',
                'listAuthorizedDevices',
                query_request
            )
            
            return result.get('devices', [])
            
        except Exception as e:
            logger.error(f"List authorized devices failed: {e}")
            return []
    
    async def update_access_policy(self, policy_id: str, policy_data: Dict[str, Any]) -> bool:
        """
        Update access policy using chaincode library.
        
        Args:
            policy_id: Policy identifier
            policy_data: Policy configuration
            
        Returns:
            Update success status
        """
        try:
            update_request = {
                'policy_id': policy_id,
                'policy_data': policy_data,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library access-control
            result = await self.blockchain_client.call_chaincode(
                'access-control',
                'updateAccessPolicy',
                update_request
            )
            
            success = result.get('success', False)
            if success:
                logger.info(f"Access policy {policy_id} updated successfully")
                # Clear related cache
                self._clear_cache()
            
            return success
            
        except Exception as e:
            logger.error(f"Update access policy failed: {e}")
            return False
    
    def _generate_cache_key(self, device_id: str, resource: str, action: str) -> str:
        """Generate cache key for access control decision."""
        return f"{device_id}:{resource}:{action}"
    
    def _is_cache_valid(self, cache_key: str) -> bool:
        """Check if cache entry is still valid."""
        if cache_key not in self.cache:
            return False
        
        entry = self.cache[cache_key]
        age = (datetime.utcnow() - entry['timestamp']).total_seconds()
        return age < entry['ttl']
    
    def _clear_device_cache(self, device_id: str):
        """Clear cache entries for a specific device."""
        keys_to_remove = [key for key in self.cache.keys() if key.startswith(f"{device_id}:")]
        for key in keys_to_remove:
            del self.cache[key]
    
    def _clear_cache(self):
        """Clear all cache entries."""
        self.cache.clear()
    
    async def _get_gateway_id(self) -> str:
        """Get the gateway ID from blockchain client."""
        return getattr(self.blockchain_client, 'gateway_id', 'unknown-gateway')
    
    async def get_access_stats(self) -> Dict[str, Any]:
        """Get access control statistics."""
        return {
            'cache_size': len(self.cache),
            'cache_hit_ratio': getattr(self, '_cache_hits', 0) / max(getattr(self, '_cache_requests', 1), 1),
            'total_checks': getattr(self, '_total_checks', 0),
            'total_grants': getattr(self, '_total_grants', 0),
            'total_denials': getattr(self, '_total_denials', 0)
        }
