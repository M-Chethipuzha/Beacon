"""
BEACON Edge Gateway - Audit Logger Integration
============================================

This module integrates with the chaincode-library audit-logging
to provide comprehensive audit trail and compliance logging.
"""

import asyncio
import logging
from typing import Dict, Any, Optional, List
from datetime import datetime
import json
import hashlib
from enum import Enum

logger = logging.getLogger(__name__)


class AuditLevel(Enum):
    """Audit log levels."""
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    CRITICAL = "critical"
    SECURITY = "security"


class AuditCategory(Enum):
    """Audit log categories."""
    DEVICE_ACCESS = "device_access"
    POLICY_ENFORCEMENT = "policy_enforcement"
    GATEWAY_MANAGEMENT = "gateway_management"
    SYSTEM_SECURITY = "system_security"
    DATA_PROCESSING = "data_processing"
    CONFIGURATION = "configuration"
    AUTHENTICATION = "authentication"


class AuditLogger:
    """Audit logging integration with chaincode library."""
    
    def __init__(self, blockchain_client):
        """Initialize audit logger with blockchain client."""
        self.blockchain_client = blockchain_client
        self.local_audit_buffer = []
        self.buffer_size = 100
        self.batch_interval = 60  # seconds
        self.privacy_enabled = True
        
        # Start background batch processor
        asyncio.create_task(self._batch_processor())
        
    async def log_audit_event(self, category: AuditCategory, level: AuditLevel,
                            event_type: str, description: str, 
                            metadata: Dict[str, Any] = None,
                            device_id: str = None, user_id: str = None) -> bool:
        """
        Log an audit event using chaincode library audit-logging.
        
        Args:
            category: Audit category
            level: Audit level
            event_type: Type of event
            description: Event description
            metadata: Additional event metadata
            device_id: Associated device ID (optional)
            user_id: Associated user ID (optional)
            
        Returns:
            Logging success status
        """
        try:
            # Prepare audit event
            audit_event = {
                'event_id': self._generate_event_id(),
                'category': category.value,
                'level': level.value,
                'event_type': event_type,
                'description': description,
                'metadata': self._sanitize_metadata(metadata or {}),
                'device_id': self._hash_device_id(device_id) if device_id and self.privacy_enabled else device_id,
                'user_id': self._hash_user_id(user_id) if user_id and self.privacy_enabled else user_id,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat(),
                'source': 'edge_gateway'
            }
            
            # Add to local buffer for batch processing
            self.local_audit_buffer.append(audit_event)
            
            # If buffer is full, flush immediately
            if len(self.local_audit_buffer) >= self.buffer_size:
                await self._flush_audit_buffer()
            
            # For critical events, send immediately
            if level in [AuditLevel.CRITICAL, AuditLevel.SECURITY]:
                await self._send_audit_event(audit_event)
            
            return True
            
        except Exception as e:
            logger.error(f"Audit logging failed: {e}")
            return False
    
    async def log_device_access(self, device_id: str, action: str, resource: str,
                              result: str, metadata: Dict[str, Any] = None) -> bool:
        """
        Log device access event.
        
        Args:
            device_id: Device identifier
            action: Access action
            resource: Accessed resource
            result: Access result (granted/denied)
            metadata: Additional metadata
            
        Returns:
            Logging success status
        """
        level = AuditLevel.INFO if result == "granted" else AuditLevel.WARNING
        description = f"Device access {result}: {action} on {resource}"
        
        access_metadata = {
            'action': action,
            'resource': resource,
            'result': result,
            **(metadata or {})
        }
        
        return await self.log_audit_event(
            AuditCategory.DEVICE_ACCESS,
            level,
            'device_access_attempt',
            description,
            access_metadata,
            device_id=device_id
        )
    
    async def log_policy_enforcement(self, policy_id: str, device_id: str,
                                   decision: str, rule_matched: str = None,
                                   metadata: Dict[str, Any] = None) -> bool:
        """
        Log policy enforcement event.
        
        Args:
            policy_id: Policy identifier
            device_id: Device identifier
            decision: Enforcement decision
            rule_matched: Matched rule (optional)
            metadata: Additional metadata
            
        Returns:
            Logging success status
        """
        level = AuditLevel.INFO if decision == "allow" else AuditLevel.WARNING
        description = f"Policy enforcement: {decision} for policy {policy_id}"
        
        policy_metadata = {
            'policy_id': policy_id,
            'decision': decision,
            'rule_matched': rule_matched,
            **(metadata or {})
        }
        
        return await self.log_audit_event(
            AuditCategory.POLICY_ENFORCEMENT,
            level,
            'policy_enforcement',
            description,
            policy_metadata,
            device_id=device_id
        )
    
    async def log_security_event(self, event_type: str, description: str,
                               severity: str = "medium", source: str = None,
                               metadata: Dict[str, Any] = None) -> bool:
        """
        Log security event.
        
        Args:
            event_type: Security event type
            description: Event description
            severity: Event severity
            source: Event source
            metadata: Additional metadata
            
        Returns:
            Logging success status
        """
        level_map = {
            'low': AuditLevel.INFO,
            'medium': AuditLevel.WARNING,
            'high': AuditLevel.ERROR,
            'critical': AuditLevel.CRITICAL
        }
        
        level = level_map.get(severity, AuditLevel.WARNING)
        
        security_metadata = {
            'severity': severity,
            'source': source,
            **(metadata or {})
        }
        
        return await self.log_audit_event(
            AuditCategory.SYSTEM_SECURITY,
            level,
            event_type,
            description,
            security_metadata
        )
    
    async def log_gateway_event(self, event_type: str, description: str,
                              metadata: Dict[str, Any] = None) -> bool:
        """
        Log gateway management event.
        
        Args:
            event_type: Gateway event type
            description: Event description
            metadata: Additional metadata
            
        Returns:
            Logging success status
        """
        return await self.log_audit_event(
            AuditCategory.GATEWAY_MANAGEMENT,
            AuditLevel.INFO,
            event_type,
            description,
            metadata
        )
    
    async def log_authentication_event(self, user_id: str, device_id: str,
                                     auth_type: str, result: str,
                                     metadata: Dict[str, Any] = None) -> bool:
        """
        Log authentication event.
        
        Args:
            user_id: User identifier
            device_id: Device identifier
            auth_type: Authentication type
            result: Authentication result
            metadata: Additional metadata
            
        Returns:
            Logging success status
        """
        level = AuditLevel.INFO if result == "success" else AuditLevel.WARNING
        description = f"Authentication {result}: {auth_type}"
        
        auth_metadata = {
            'auth_type': auth_type,
            'result': result,
            **(metadata or {})
        }
        
        return await self.log_audit_event(
            AuditCategory.AUTHENTICATION,
            level,
            'authentication_attempt',
            description,
            auth_metadata,
            device_id=device_id,
            user_id=user_id
        )
    
    async def query_audit_logs(self, criteria: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Query audit logs from chaincode library.
        
        Args:
            criteria: Query criteria (category, level, date range, etc.)
            
        Returns:
            List of matching audit logs
        """
        try:
            query_request = {
                'criteria': criteria,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library audit-logging
            result = await self.blockchain_client.call_chaincode(
                'audit-logging',
                'queryAuditLogs',
                query_request
            )
            
            return result.get('audit_logs', [])
            
        except Exception as e:
            logger.error(f"Audit log query failed: {e}")
            return []
    
    async def get_audit_summary(self, date_range: Dict[str, str]) -> Dict[str, Any]:
        """
        Get audit summary for a date range.
        
        Args:
            date_range: Date range with 'start' and 'end' timestamps
            
        Returns:
            Audit summary statistics
        """
        try:
            summary_request = {
                'date_range': date_range,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library audit-logging
            result = await self.blockchain_client.call_chaincode(
                'audit-logging',
                'getAuditSummary',
                summary_request
            )
            
            return result.get('summary', {})
            
        except Exception as e:
            logger.error(f"Audit summary query failed: {e}")
            return {}
    
    async def export_audit_logs(self, criteria: Dict[str, Any], 
                              format: str = "json") -> Optional[str]:
        """
        Export audit logs in specified format.
        
        Args:
            criteria: Export criteria
            format: Export format (json, csv, xml)
            
        Returns:
            Export data or None if failed
        """
        try:
            export_request = {
                'criteria': criteria,
                'format': format,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library audit-logging
            result = await self.blockchain_client.call_chaincode(
                'audit-logging',
                'exportAuditLogs',
                export_request
            )
            
            return result.get('export_data')
            
        except Exception as e:
            logger.error(f"Audit log export failed: {e}")
            return None
    
    async def _send_audit_event(self, audit_event: Dict[str, Any]) -> bool:
        """Send individual audit event to chaincode."""
        try:
            # Call chaincode library audit-logging
            result = await self.blockchain_client.call_chaincode(
                'audit-logging',
                'logAuditEvent',
                audit_event
            )
            
            return result.get('success', False)
            
        except Exception as e:
            logger.error(f"Send audit event failed: {e}")
            return False
    
    async def _flush_audit_buffer(self) -> bool:
        """Flush local audit buffer to chaincode."""
        if not self.local_audit_buffer:
            return True
        
        try:
            batch_request = {
                'audit_events': self.local_audit_buffer.copy(),
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library audit-logging
            result = await self.blockchain_client.call_chaincode(
                'audit-logging',
                'logAuditBatch',
                batch_request
            )
            
            success = result.get('success', False)
            if success:
                # Clear the buffer
                self.local_audit_buffer.clear()
                logger.debug(f"Flushed {len(batch_request['audit_events'])} audit events")
            else:
                logger.error(f"Audit batch flush failed: {result.get('error', 'unknown')}")
            
            return success
            
        except Exception as e:
            logger.error(f"Audit buffer flush failed: {e}")
            return False
    
    async def _batch_processor(self):
        """Background task to periodically flush audit buffer."""
        while True:
            try:
                await asyncio.sleep(self.batch_interval)
                if self.local_audit_buffer:
                    await self._flush_audit_buffer()
            except Exception as e:
                logger.error(f"Batch processor error: {e}")
    
    def _generate_event_id(self) -> str:
        """Generate unique event ID."""
        import uuid
        return str(uuid.uuid4())
    
    def _hash_device_id(self, device_id: str) -> str:
        """Hash device ID for privacy."""
        if not device_id:
            return None
        salt = getattr(self, 'device_salt', 'default_salt')
        return hashlib.sha256(f"{device_id}{salt}".encode()).hexdigest()[:16]
    
    def _hash_user_id(self, user_id: str) -> str:
        """Hash user ID for privacy."""
        if not user_id:
            return None
        salt = getattr(self, 'user_salt', 'default_salt')
        return hashlib.sha256(f"{user_id}{salt}".encode()).hexdigest()[:16]
    
    def _sanitize_metadata(self, metadata: Dict[str, Any]) -> Dict[str, Any]:
        """Sanitize metadata to remove sensitive information."""
        sanitized = {}
        sensitive_keys = ['password', 'token', 'key', 'secret', 'credential']
        
        for key, value in metadata.items():
            if any(sensitive in key.lower() for sensitive in sensitive_keys):
                sanitized[key] = '[REDACTED]'
            else:
                sanitized[key] = value
        
        return sanitized
    
    async def _get_gateway_id(self) -> str:
        """Get the gateway ID from blockchain client."""
        return getattr(self.blockchain_client, 'gateway_id', 'unknown-gateway')
    
    def set_privacy_mode(self, enabled: bool):
        """Enable or disable privacy mode for ID hashing."""
        self.privacy_enabled = enabled
        logger.info(f"Privacy mode {'enabled' if enabled else 'disabled'}")
    
    def get_buffer_status(self) -> Dict[str, Any]:
        """Get audit buffer status."""
        return {
            'buffer_size': len(self.local_audit_buffer),
            'buffer_limit': self.buffer_size,
            'batch_interval': self.batch_interval,
            'privacy_enabled': self.privacy_enabled
        }
    
    async def get_audit_stats(self) -> Dict[str, Any]:
        """Get audit logging statistics."""
        return {
            'buffer_size': len(self.local_audit_buffer),
            'total_events_logged': getattr(self, '_total_events', 0),
            'total_batches_sent': getattr(self, '_total_batches', 0),
            'failed_events': getattr(self, '_failed_events', 0),
            'privacy_mode': self.privacy_enabled
        }
