"""
Policy Enforcement Engine

Handles real-time enforcement of access control policies for IoT devices.
Integrates with chaincode library for policy evaluation and decision logging.
"""

import asyncio
import logging
import hashlib
from typing import Dict, List, Optional, Any, Tuple
from datetime import datetime
from dataclasses import dataclass
from enum import Enum

from .cache import PolicyCache, Policy

logger = logging.getLogger(__name__)

class AccessDecision(Enum):
    """Access control decision types"""
    ALLOW = "allow"
    DENY = "deny"
    UNKNOWN = "unknown"

@dataclass
class AccessRequest:
    """Represents an access control request"""
    device_id: str
    device_type: str
    action: str
    resource: str
    timestamp: datetime
    source_ip: Optional[str] = None
    protocol: str = "unknown"
    additional_attributes: Optional[Dict[str, Any]] = None
    
    def get_hashed_device_id(self, gateway_salt: str) -> str:
        """Generate privacy-preserving hashed device ID"""
        combined = f"{self.device_id}:{gateway_salt}"
        return hashlib.sha256(combined.encode()).hexdigest()

@dataclass
class AccessResult:
    """Represents the result of access control evaluation"""
    decision: AccessDecision
    policy_id: Optional[str]
    rule_matched: Optional[str]
    reason: str
    confidence: float = 1.0
    processing_time_ms: float = 0.0
    additional_metadata: Optional[Dict[str, Any]] = None

class PolicyEnforcer:
    """
    Real-time policy enforcement engine.
    
    Features:
    - Fast in-memory policy evaluation
    - Privacy-preserving device identification
    - Audit logging integration
    - Fallback mechanisms for offline operation
    """
    
    def __init__(self, 
                 policy_cache: PolicyCache,
                 gateway_id: str,
                 default_decision: AccessDecision = AccessDecision.DENY):
        """
        Initialize policy enforcement engine.
        
        Args:
            policy_cache: Policy cache instance
            gateway_id: Unique gateway identifier for privacy hashing
            default_decision: Default decision when no policies match
        """
        self.policy_cache = policy_cache
        self.gateway_id = gateway_id
        self.default_decision = default_decision
        
        # Generate gateway-specific salt for device ID hashing
        self.gateway_salt = hashlib.sha256(f"beacon-gateway-{gateway_id}".encode()).hexdigest()[:16]
        
        # In-memory policy cache for fast lookups
        self._cached_policies: List[Policy] = []
        self._cache_last_updated: Optional[datetime] = None
        self._cache_refresh_interval = 60  # seconds
        
        # Statistics
        self._stats = {
            "total_requests": 0,
            "allow_decisions": 0,
            "deny_decisions": 0,
            "unknown_decisions": 0,
            "policy_cache_hits": 0,
            "policy_cache_misses": 0,
            "average_processing_time_ms": 0.0
        }
        
    async def evaluate_access_request(self, request: AccessRequest) -> AccessResult:
        """
        Evaluate an access control request against cached policies.
        
        Args:
            request: Access request to evaluate
            
        Returns:
            Access control decision result
        """
        start_time = datetime.now()
        
        try:
            # Refresh policy cache if needed
            await self._refresh_policy_cache_if_needed()
            
            # Generate privacy-preserving device ID
            hashed_device_id = request.get_hashed_device_id(self.gateway_salt)
            
            # Evaluate against cached policies
            result = await self._evaluate_policies(request, hashed_device_id)
            
            # Calculate processing time
            processing_time = (datetime.now() - start_time).total_seconds() * 1000
            result.processing_time_ms = processing_time
            
            # Update statistics
            await self._update_stats(result)
            
            # Log decision (with privacy preservation)
            await self._log_access_decision(request, result, hashed_device_id)
            
            logger.debug(
                f"Access decision for device {request.device_id[:8]}***: "
                f"{result.decision.value} ({processing_time:.1f}ms)"
            )
            
            return result
            
        except Exception as e:
            logger.error(f"Error evaluating access request: {e}")
            
            # Return safe default decision
            return AccessResult(
                decision=self.default_decision,
                policy_id=None,
                rule_matched=None,
                reason=f"Error during policy evaluation: {str(e)}",
                processing_time_ms=(datetime.now() - start_time).total_seconds() * 1000
            )
    
    async def _refresh_policy_cache_if_needed(self):
        """Refresh in-memory policy cache if outdated"""
        now = datetime.now()
        
        if (not self._cache_last_updated or 
            (now - self._cache_last_updated).total_seconds() > self._cache_refresh_interval):
            
            logger.debug("Refreshing in-memory policy cache")
            
            try:
                self._cached_policies = await self.policy_cache.get_all_policies(
                    enabled_only=True,
                    exclude_expired=True
                )
                self._cache_last_updated = now
                
                logger.debug(f"Loaded {len(self._cached_policies)} policies into memory")
                
            except Exception as e:
                logger.error(f"Failed to refresh policy cache: {e}")
    
    async def _evaluate_policies(self, 
                                request: AccessRequest, 
                                hashed_device_id: str) -> AccessResult:
        """
        Evaluate request against all applicable policies.
        
        Args:
            request: Original access request
            hashed_device_id: Privacy-preserving device identifier
            
        Returns:
            Access control result
        """
        if not self._cached_policies:
            return AccessResult(
                decision=self.default_decision,
                policy_id=None,
                rule_matched=None,
                reason="No policies available for evaluation"
            )
        
        # Evaluate policies in priority order (highest first)
        for policy in self._cached_policies:
            try:
                result = await self._evaluate_single_policy(policy, request, hashed_device_id)
                if result:
                    self._stats["policy_cache_hits"] += 1
                    return result
                    
            except Exception as e:
                logger.warning(f"Error evaluating policy {policy.id}: {e}")
                continue
        
        # No policies matched
        self._stats["policy_cache_misses"] += 1
        return AccessResult(
            decision=self.default_decision,
            policy_id=None,
            rule_matched=None,
            reason="No matching policies found"
        )
    
    async def _evaluate_single_policy(self, 
                                     policy: Policy, 
                                     request: AccessRequest,
                                     hashed_device_id: str) -> Optional[AccessResult]:
        """
        Evaluate request against a single policy.
        
        Args:
            policy: Policy to evaluate
            request: Access request
            hashed_device_id: Privacy-preserving device ID
            
        Returns:
            AccessResult if policy matches, None otherwise
        """
        rules = policy.rules
        
        # Check if policy applies to this request
        if not await self._policy_applies(rules, request, hashed_device_id):
            return None
        
        # Evaluate policy conditions
        decision = await self._evaluate_policy_conditions(rules, request, hashed_device_id)
        
        return AccessResult(
            decision=decision,
            policy_id=policy.id,
            rule_matched=policy.name,
            reason=f"Matched policy: {policy.name}",
            confidence=1.0
        )
    
    async def _policy_applies(self, 
                             rules: Dict[str, Any], 
                             request: AccessRequest,
                             hashed_device_id: str) -> bool:
        """
        Check if a policy applies to the given request.
        
        Args:
            rules: Policy rules dictionary
            request: Access request
            hashed_device_id: Privacy-preserving device ID
            
        Returns:
            True if policy applies
        """
        # Check device type filter
        if "device_types" in rules:
            device_types = rules["device_types"]
            if isinstance(device_types, list) and request.device_type not in device_types:
                return False
        
        # Check action filter
        if "actions" in rules:
            actions = rules["actions"]
            if isinstance(actions, list) and request.action not in actions:
                return False
        
        # Check resource filter
        if "resources" in rules:
            resources = rules["resources"]
            if isinstance(resources, list) and request.resource not in resources:
                return False
        
        # Check time-based conditions
        if "time_restrictions" in rules:
            if not await self._check_time_restrictions(rules["time_restrictions"]):
                return False
        
        # Check device-specific conditions (using hashed ID)
        if "device_conditions" in rules:
            if not await self._check_device_conditions(
                rules["device_conditions"], 
                hashed_device_id, 
                request
            ):
                return False
        
        return True
    
    async def _evaluate_policy_conditions(self, 
                                         rules: Dict[str, Any], 
                                         request: AccessRequest,
                                         hashed_device_id: str) -> AccessDecision:
        """
        Evaluate policy conditions to make access decision.
        
        Args:
            rules: Policy rules dictionary
            request: Access request
            hashed_device_id: Privacy-preserving device ID
            
        Returns:
            Access decision
        """
        # Default decision from policy
        default_action = rules.get("default_action", "deny")
        
        # Check explicit allow/deny rules
        if "allow_rules" in rules:
            for rule in rules["allow_rules"]:
                if await self._evaluate_rule_condition(rule, request, hashed_device_id):
                    return AccessDecision.ALLOW
        
        if "deny_rules" in rules:
            for rule in rules["deny_rules"]:
                if await self._evaluate_rule_condition(rule, request, hashed_device_id):
                    return AccessDecision.DENY
        
        # Return default action
        return AccessDecision.ALLOW if default_action == "allow" else AccessDecision.DENY
    
    async def _evaluate_rule_condition(self, 
                                      rule: Dict[str, Any], 
                                      request: AccessRequest,
                                      hashed_device_id: str) -> bool:
        """
        Evaluate a single rule condition.
        
        Args:
            rule: Rule condition dictionary
            request: Access request
            hashed_device_id: Privacy-preserving device ID
            
        Returns:
            True if condition matches
        """
        # Simple condition evaluation - can be extended
        
        # IP address checks
        if "source_ip" in rule and request.source_ip:
            if request.source_ip != rule["source_ip"]:
                return False
        
        # Protocol checks
        if "protocol" in rule:
            if request.protocol != rule["protocol"]:
                return False
        
        # Custom attribute checks
        if "attributes" in rule and request.additional_attributes:
            for key, expected_value in rule["attributes"].items():
                if request.additional_attributes.get(key) != expected_value:
                    return False
        
        return True
    
    async def _check_time_restrictions(self, time_rules: Dict[str, Any]) -> bool:
        """Check if current time satisfies time restrictions"""
        now = datetime.now()
        
        # Check allowed hours
        if "allowed_hours" in time_rules:
            allowed_hours = time_rules["allowed_hours"]
            if now.hour not in allowed_hours:
                return False
        
        # Check allowed days of week (0 = Monday, 6 = Sunday)
        if "allowed_weekdays" in time_rules:
            allowed_weekdays = time_rules["allowed_weekdays"]
            if now.weekday() not in allowed_weekdays:
                return False
        
        return True
    
    async def _check_device_conditions(self, 
                                      device_rules: Dict[str, Any],
                                      hashed_device_id: str,
                                      request: AccessRequest) -> bool:
        """Check device-specific conditions"""
        
        # Check device whitelist (using hashed IDs)
        if "allowed_devices" in device_rules:
            allowed_devices = device_rules["allowed_devices"]
            if hashed_device_id not in allowed_devices:
                return False
        
        # Check device blacklist (using hashed IDs)
        if "denied_devices" in device_rules:
            denied_devices = device_rules["denied_devices"]
            if hashed_device_id in denied_devices:
                return False
        
        return True
    
    async def _log_access_decision(self, 
                                  request: AccessRequest,
                                  result: AccessResult,
                                  hashed_device_id: str):
        """
        Log access decision for audit purposes.
        
        Args:
            request: Original access request
            result: Access decision result
            hashed_device_id: Privacy-preserving device ID
        """
        try:
            # TODO: Integrate with chaincode-library/audit-logging
            # This would call the audit logging chaincode to record the decision
            
            audit_entry = {
                "timestamp": request.timestamp.isoformat(),
                "gateway_id": self.gateway_id,
                "hashed_device_id": hashed_device_id,  # Privacy-preserving
                "device_type": request.device_type,
                "action": request.action,
                "resource": request.resource,
                "decision": result.decision.value,
                "policy_id": result.policy_id,
                "reason": result.reason,
                "processing_time_ms": result.processing_time_ms
            }
            
            # For now, just log locally
            logger.info(f"Access decision logged: {audit_entry}")
            
        except Exception as e:
            logger.error(f"Failed to log access decision: {e}")
    
    async def _update_stats(self, result: AccessResult):
        """Update enforcement statistics"""
        self._stats["total_requests"] += 1
        
        if result.decision == AccessDecision.ALLOW:
            self._stats["allow_decisions"] += 1
        elif result.decision == AccessDecision.DENY:
            self._stats["deny_decisions"] += 1
        else:
            self._stats["unknown_decisions"] += 1
        
        # Update average processing time
        total_requests = self._stats["total_requests"]
        current_avg = self._stats["average_processing_time_ms"]
        new_avg = ((current_avg * (total_requests - 1)) + result.processing_time_ms) / total_requests
        self._stats["average_processing_time_ms"] = new_avg
    
    def get_enforcement_stats(self) -> Dict[str, Any]:
        """Get enforcement statistics"""
        return {
            **self._stats,
            "cached_policies_count": len(self._cached_policies),
            "cache_last_updated": self._cache_last_updated.isoformat() if self._cache_last_updated else None,
            "gateway_id": self.gateway_id
        }
    
    async def reload_policies(self):
        """Force reload of policies from cache"""
        logger.info("Force reloading policies from cache")
        self._cache_last_updated = None
        await self._refresh_policy_cache_if_needed()
