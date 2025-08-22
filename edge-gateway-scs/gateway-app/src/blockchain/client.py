"""
Blockchain Client

Main client for communicating with the I&O SCS blockchain.
Handles gateway registration, policy synchronization, and chaincode integration.
"""

import asyncio
import aiohttp
import logging
import json
from typing import Dict, List, Optional, Any
from datetime import datetime
from dataclasses import dataclass

from ..discovery.io_scs_discovery import IOSCSDiscovery

logger = logging.getLogger(__name__)

@dataclass
class GatewayInfo:
    """Gateway registration information"""
    gateway_id: str
    name: str
    location: str
    capabilities: List[str]
    public_key: str
    network_info: Dict[str, Any]
    metadata: Dict[str, Any]

@dataclass
class PolicySyncResult:
    """Result of policy synchronization"""
    success: bool
    policies_updated: int
    policies_added: int
    policies_removed: int
    last_sync_time: datetime
    error_message: Optional[str] = None

class BlockchainClient:
    """
    Client for I&O SCS blockchain communication.
    
    Features:
    - Gateway registration and lifecycle management
    - Policy synchronization
    - Chaincode interaction
    - Privacy-preserving operations
    """
    
    def __init__(self, 
                 discovery_service: IOSCSDiscovery,
                 gateway_info: GatewayInfo,
                 sync_interval: int = 300):
        """
        Initialize blockchain client.
        
        Args:
            discovery_service: I&O SCS discovery service
            gateway_info: Gateway registration information
            sync_interval: Policy sync interval in seconds
        """
        self.discovery = discovery_service
        self.gateway_info = gateway_info
        self.sync_interval = sync_interval
        
        # Registration state
        self.registered = False
        self.registration_token: Optional[str] = None
        
        # Synchronization state
        self.last_policy_sync: Optional[datetime] = None
        self.sync_in_progress = False
        self._sync_lock = asyncio.Lock()
        
        # Background tasks
        self._sync_task: Optional[asyncio.Task] = None
        self._heartbeat_task: Optional[asyncio.Task] = None
        
        # Statistics
        self.stats = {
            "registration_attempts": 0,
            "successful_registrations": 0,
            "policy_sync_count": 0,
            "failed_sync_count": 0,
            "last_blockchain_contact": None,
            "chaincode_calls": 0,
            "chaincode_errors": 0
        }
    
    async def start(self):
        """Start blockchain client operations"""
        logger.info("Starting blockchain client")
        
        try:
            # Register with I&O SCS
            await self._register_gateway()
            
            # Start background tasks
            self._sync_task = asyncio.create_task(self._policy_sync_loop())
            self._heartbeat_task = asyncio.create_task(self._heartbeat_loop())
            
            logger.info("Blockchain client started successfully")
            
        except Exception as e:
            logger.error(f"Failed to start blockchain client: {e}")
            raise
    
    async def stop(self):
        """Stop blockchain client operations"""
        logger.info("Stopping blockchain client")
        
        # Cancel background tasks
        if self._sync_task:
            self._sync_task.cancel()
            try:
                await self._sync_task
            except asyncio.CancelledError:
                pass
        
        if self._heartbeat_task:
            self._heartbeat_task.cancel()
            try:
                await self._heartbeat_task
            except asyncio.CancelledError:
                pass
        
        # Unregister gateway if registered
        if self.registered:
            await self._unregister_gateway()
        
        logger.info("Blockchain client stopped")
    
    async def _register_gateway(self) -> bool:
        """
        Register gateway with I&O SCS blockchain.
        
        Returns:
            True if registration successful
        """
        try:
            self.stats["registration_attempts"] += 1
            logger.info(f"Registering gateway: {self.gateway_info.gateway_id}")
            
            # Prepare registration payload
            registration_data = {
                "gateway_id": self.gateway_info.gateway_id,
                "name": self.gateway_info.name,
                "location": self.gateway_info.location,
                "capabilities": self.gateway_info.capabilities,
                "public_key": self.gateway_info.public_key,
                "network_info": self.gateway_info.network_info,
                "metadata": self.gateway_info.metadata,
                "timestamp": datetime.now().isoformat()
            }
            
            # Call gateway-management chaincode via I&O SCS API
            response = await self.discovery.make_request(
                endpoint="/chaincode/gateway-management/register",
                method="POST",
                data=registration_data
            )
            
            if response and response.get("success"):
                self.registered = True
                self.registration_token = response.get("token")
                self.stats["successful_registrations"] += 1
                self.stats["last_blockchain_contact"] = datetime.now().isoformat()
                
                logger.info("Gateway registered successfully")
                return True
            else:
                error_msg = response.get("error", "Unknown error") if response else "No response"
                logger.error(f"Gateway registration failed: {error_msg}")
                return False
                
        except Exception as e:
            logger.error(f"Error during gateway registration: {e}")
            return False
    
    async def _unregister_gateway(self) -> bool:
        """
        Unregister gateway from I&O SCS blockchain.
        
        Returns:
            True if unregistration successful
        """
        try:
            if not self.registered:
                return True
                
            logger.info(f"Unregistering gateway: {self.gateway_info.gateway_id}")
            
            unregistration_data = {
                "gateway_id": self.gateway_info.gateway_id,
                "token": self.registration_token,
                "timestamp": datetime.now().isoformat()
            }
            
            response = await self.discovery.make_request(
                endpoint="/chaincode/gateway-management/unregister",
                method="POST",
                data=unregistration_data
            )
            
            if response and response.get("success"):
                self.registered = False
                self.registration_token = None
                logger.info("Gateway unregistered successfully")
                return True
            else:
                error_msg = response.get("error", "Unknown error") if response else "No response"
                logger.warning(f"Gateway unregistration failed: {error_msg}")
                return False
                
        except Exception as e:
            logger.error(f"Error during gateway unregistration: {e}")
            return False
    
    async def sync_policies(self) -> PolicySyncResult:
        """
        Synchronize policies from I&O SCS blockchain.
        
        Returns:
            Policy synchronization result
        """
        async with self._sync_lock:
            if self.sync_in_progress:
                logger.debug("Policy sync already in progress")
                return PolicySyncResult(
                    success=False,
                    policies_updated=0,
                    policies_added=0,
                    policies_removed=0,
                    last_sync_time=datetime.now(),
                    error_message="Sync already in progress"
                )
            
            self.sync_in_progress = True
            
        try:
            logger.debug("Starting policy synchronization")
            start_time = datetime.now()
            
            # Get last sync timestamp for incremental updates
            since_timestamp = self.last_policy_sync.isoformat() if self.last_policy_sync else None
            
            # Request policy updates from I&O SCS
            sync_data = {
                "gateway_id": self.gateway_info.gateway_id,
                "token": self.registration_token,
                "since": since_timestamp,
                "include_disabled": False
            }
            
            response = await self.discovery.make_request(
                endpoint="/chaincode/policy-enforcement/sync",
                method="POST",
                data=sync_data
            )
            
            if not response or not response.get("success"):
                error_msg = response.get("error", "No response") if response else "No response"
                logger.error(f"Policy sync failed: {error_msg}")
                self.stats["failed_sync_count"] += 1
                
                return PolicySyncResult(
                    success=False,
                    policies_updated=0,
                    policies_added=0,
                    policies_removed=0,
                    last_sync_time=start_time,
                    error_message=error_msg
                )
            
            # Process policy updates
            policies_data = response.get("policies", [])
            removed_policies = response.get("removed_policies", [])
            
            policies_added = 0
            policies_updated = 0
            policies_removed = len(removed_policies)
            
            # TODO: Update local policy cache with received policies
            # This would integrate with policy.cache.PolicyCache
            
            for policy_data in policies_data:
                # Determine if this is a new policy or update
                # For now, count as updates
                policies_updated += 1
            
            # Update sync state
            self.last_policy_sync = start_time
            self.stats["policy_sync_count"] += 1
            self.stats["last_blockchain_contact"] = start_time.isoformat()
            
            logger.info(f"Policy sync completed: {policies_updated} updated, {policies_removed} removed")
            
            return PolicySyncResult(
                success=True,
                policies_updated=policies_updated,
                policies_added=policies_added,
                policies_removed=policies_removed,
                last_sync_time=start_time
            )
            
        except Exception as e:
            logger.error(f"Error during policy synchronization: {e}")
            self.stats["failed_sync_count"] += 1
            
            return PolicySyncResult(
                success=False,
                policies_updated=0,
                policies_added=0,
                policies_removed=0,
                last_sync_time=datetime.now(),
                error_message=str(e)
            )
            
        finally:
            self.sync_in_progress = False
    
    async def call_access_control_chaincode(self, 
                                           operation: str, 
                                           data: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """
        Call access-control chaincode on I&O SCS.
        
        Args:
            operation: Chaincode operation (e.g., "check_access", "log_decision")
            data: Operation parameters
            
        Returns:
            Chaincode response or None if failed
        """
        try:
            self.stats["chaincode_calls"] += 1
            
            request_data = {
                "gateway_id": self.gateway_info.gateway_id,
                "token": self.registration_token,
                "operation": operation,
                "data": data,
                "timestamp": datetime.now().isoformat()
            }
            
            response = await self.discovery.make_request(
                endpoint="/chaincode/access-control/invoke",
                method="POST",
                data=request_data
            )
            
            if response and response.get("success"):
                self.stats["last_blockchain_contact"] = datetime.now().isoformat()
                return response.get("result")
            else:
                error_msg = response.get("error", "Unknown error") if response else "No response"
                logger.warning(f"Access control chaincode call failed: {error_msg}")
                self.stats["chaincode_errors"] += 1
                return None
                
        except Exception as e:
            logger.error(f"Error calling access control chaincode: {e}")
            self.stats["chaincode_errors"] += 1
            return None
    
    async def call_audit_logging_chaincode(self, 
                                          audit_data: Dict[str, Any]) -> bool:
        """
        Log audit event using audit-logging chaincode.
        
        Args:
            audit_data: Audit event data
            
        Returns:
            True if logged successfully
        """
        try:
            self.stats["chaincode_calls"] += 1
            
            request_data = {
                "gateway_id": self.gateway_info.gateway_id,
                "token": self.registration_token,
                "audit_data": audit_data,
                "timestamp": datetime.now().isoformat()
            }
            
            response = await self.discovery.make_request(
                endpoint="/chaincode/audit-logging/log",
                method="POST",
                data=request_data
            )
            
            if response and response.get("success"):
                self.stats["last_blockchain_contact"] = datetime.now().isoformat()
                return True
            else:
                error_msg = response.get("error", "Unknown error") if response else "No response"
                logger.warning(f"Audit logging failed: {error_msg}")
                self.stats["chaincode_errors"] += 1
                return False
                
        except Exception as e:
            logger.error(f"Error calling audit logging chaincode: {e}")
            self.stats["chaincode_errors"] += 1
            return False
    
    async def register_device(self, 
                             device_id: str, 
                             device_info: Dict[str, Any]) -> bool:
        """
        Register device using device-registry chaincode.
        
        Args:
            device_id: Local device identifier
            device_info: Device metadata
            
        Returns:
            True if registered successfully
        """
        try:
            # Generate privacy-preserving hashed device ID
            import hashlib
            gateway_salt = hashlib.sha256(f"beacon-gateway-{self.gateway_info.gateway_id}".encode()).hexdigest()[:16]
            hashed_device_id = hashlib.sha256(f"{device_id}:{gateway_salt}".encode()).hexdigest()
            
            registration_data = {
                "gateway_id": self.gateway_info.gateway_id,
                "token": self.registration_token,
                "hashed_device_id": hashed_device_id,
                "device_type": device_info.get("type", "unknown"),
                "capabilities": device_info.get("capabilities", []),
                "metadata": {
                    "registered_at": datetime.now().isoformat(),
                    **device_info.get("metadata", {})
                }
            }
            
            response = await self.discovery.make_request(
                endpoint="/chaincode/device-registry/register",
                method="POST",
                data=registration_data
            )
            
            if response and response.get("success"):
                logger.info(f"Device registered: {device_id} -> {hashed_device_id[:16]}***")
                return True
            else:
                error_msg = response.get("error", "Unknown error") if response else "No response"
                logger.error(f"Device registration failed: {error_msg}")
                return False
                
        except Exception as e:
            logger.error(f"Error registering device {device_id}: {e}")
            return False
    
    async def _policy_sync_loop(self):
        """Background task for periodic policy synchronization"""
        while True:
            try:
                if self.registered:
                    await self.sync_policies()
                
                await asyncio.sleep(self.sync_interval)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Policy sync loop error: {e}")
                await asyncio.sleep(30)  # Brief delay before retry
    
    async def _heartbeat_loop(self):
        """Background task for gateway heartbeat"""
        heartbeat_interval = 60  # seconds
        
        while True:
            try:
                if self.registered:
                    await self._send_heartbeat()
                
                await asyncio.sleep(heartbeat_interval)
                
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Heartbeat loop error: {e}")
                await asyncio.sleep(10)  # Brief delay before retry
    
    async def _send_heartbeat(self):
        """Send heartbeat to I&O SCS"""
        try:
            heartbeat_data = {
                "gateway_id": self.gateway_info.gateway_id,
                "token": self.registration_token,
                "timestamp": datetime.now().isoformat(),
                "status": "online"
            }
            
            response = await self.discovery.make_request(
                endpoint="/chaincode/gateway-management/heartbeat",
                method="POST",
                data=heartbeat_data
            )
            
            if response and response.get("success"):
                self.stats["last_blockchain_contact"] = datetime.now().isoformat()
                logger.debug("Heartbeat sent successfully")
            else:
                logger.warning("Heartbeat failed")
                
        except Exception as e:
            logger.debug(f"Heartbeat error: {e}")
    
    def get_blockchain_stats(self) -> Dict[str, Any]:
        """Get blockchain client statistics"""
        return {
            **self.stats,
            "registered": self.registered,
            "gateway_id": self.gateway_info.gateway_id,
            "last_policy_sync": self.last_policy_sync.isoformat() if self.last_policy_sync else None,
            "sync_in_progress": self.sync_in_progress
        }
