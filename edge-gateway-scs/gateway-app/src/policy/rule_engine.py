"""
BEACON Edge Gateway - Rule Engine Integration
===========================================

This module integrates with the chaincode-library policy-enforcement
to provide dynamic rule evaluation and policy execution.
"""

import asyncio
import logging
from typing import Dict, Any, Optional, List, Callable
from datetime import datetime
import json
import re

logger = logging.getLogger(__name__)


class RuleEngine:
    """Rule engine integration with chaincode library policy-enforcement."""
    
    def __init__(self, blockchain_client):
        """Initialize rule engine with blockchain client."""
        self.blockchain_client = blockchain_client
        self.rules_cache = {}
        self.rule_handlers = {}
        self.evaluation_cache = {}
        self.cache_ttl = 180  # 3 minutes cache TTL
        
        # Register built-in rule handlers
        self._register_builtin_handlers()
        
    async def evaluate_rules(self, event_type: str, event_data: Dict[str, Any], 
                           context: Dict[str, Any] = None) -> List[Dict[str, Any]]:
        """
        Evaluate rules for a given event using chaincode policy-enforcement.
        
        Args:
            event_type: Type of event (device_connect, data_receive, etc.)
            event_data: Event data and parameters
            context: Additional context information
            
        Returns:
            List of rule evaluation results and actions
        """
        try:
            # Prepare evaluation request
            evaluation_request = {
                'event_type': event_type,
                'event_data': event_data,
                'context': context or {},
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Check cache first
            cache_key = self._generate_evaluation_cache_key(event_type, event_data)
            if self._is_evaluation_cache_valid(cache_key):
                logger.debug(f"Rule evaluation cache hit for {cache_key}")
                return self.evaluation_cache[cache_key]['results']
            
            # Call chaincode library policy-enforcement
            result = await self.blockchain_client.call_chaincode(
                'policy-enforcement',
                'evaluateRules',
                evaluation_request
            )
            
            rule_results = result.get('rule_results', [])
            
            # Cache the results
            self.evaluation_cache[cache_key] = {
                'results': rule_results,
                'timestamp': datetime.utcnow(),
                'ttl': self.cache_ttl
            }
            
            # Execute local actions for matched rules
            await self._execute_rule_actions(rule_results)
            
            logger.info(f"Evaluated {len(rule_results)} rules for event {event_type}")
            return rule_results
            
        except Exception as e:
            logger.error(f"Rule evaluation failed: {e}")
            return []
    
    async def load_rules(self, rule_category: str = None) -> bool:
        """
        Load rules from chaincode library policy-enforcement.
        
        Args:
            rule_category: Optional category filter
            
        Returns:
            Load success status
        """
        try:
            load_request = {
                'gateway_id': await self._get_gateway_id(),
                'category': rule_category
            }
            
            # Call chaincode library policy-enforcement
            result = await self.blockchain_client.call_chaincode(
                'policy-enforcement',
                'getRules',
                load_request
            )
            
            rules = result.get('rules', [])
            
            # Update local rules cache
            for rule in rules:
                rule_id = rule.get('rule_id')
                if rule_id:
                    self.rules_cache[rule_id] = {
                        'rule': rule,
                        'timestamp': datetime.utcnow()
                    }
            
            logger.info(f"Loaded {len(rules)} rules from chaincode")
            return True
            
        except Exception as e:
            logger.error(f"Rule loading failed: {e}")
            return False
    
    async def create_rule(self, rule_data: Dict[str, Any]) -> str:
        """
        Create a new rule using chaincode library.
        
        Args:
            rule_data: Rule definition and configuration
            
        Returns:
            Created rule ID or None if failed
        """
        try:
            create_request = {
                'rule_data': rule_data,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library policy-enforcement
            result = await self.blockchain_client.call_chaincode(
                'policy-enforcement',
                'createRule',
                create_request
            )
            
            rule_id = result.get('rule_id')
            if rule_id:
                logger.info(f"Rule {rule_id} created successfully")
                # Reload rules to get the new rule
                await self.load_rules()
            
            return rule_id
            
        except Exception as e:
            logger.error(f"Rule creation failed: {e}")
            return None
    
    async def update_rule(self, rule_id: str, rule_data: Dict[str, Any]) -> bool:
        """
        Update an existing rule using chaincode library.
        
        Args:
            rule_id: Rule identifier
            rule_data: Updated rule configuration
            
        Returns:
            Update success status
        """
        try:
            update_request = {
                'rule_id': rule_id,
                'rule_data': rule_data,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library policy-enforcement
            result = await self.blockchain_client.call_chaincode(
                'policy-enforcement',
                'updateRule',
                update_request
            )
            
            success = result.get('success', False)
            if success:
                logger.info(f"Rule {rule_id} updated successfully")
                # Update local cache
                if rule_id in self.rules_cache:
                    self.rules_cache[rule_id]['rule'].update(rule_data)
                    self.rules_cache[rule_id]['timestamp'] = datetime.utcnow()
                # Clear evaluation cache
                self._clear_evaluation_cache()
            
            return success
            
        except Exception as e:
            logger.error(f"Rule update failed: {e}")
            return False
    
    async def delete_rule(self, rule_id: str) -> bool:
        """
        Delete a rule using chaincode library.
        
        Args:
            rule_id: Rule identifier
            
        Returns:
            Deletion success status
        """
        try:
            delete_request = {
                'rule_id': rule_id,
                'gateway_id': await self._get_gateway_id(),
                'timestamp': datetime.utcnow().isoformat()
            }
            
            # Call chaincode library policy-enforcement
            result = await self.blockchain_client.call_chaincode(
                'policy-enforcement',
                'deleteRule',
                delete_request
            )
            
            success = result.get('success', False)
            if success:
                logger.info(f"Rule {rule_id} deleted successfully")
                # Remove from local cache
                if rule_id in self.rules_cache:
                    del self.rules_cache[rule_id]
                # Clear evaluation cache
                self._clear_evaluation_cache()
            
            return success
            
        except Exception as e:
            logger.error(f"Rule deletion failed: {e}")
            return False
    
    async def get_rule_execution_history(self, rule_id: str = None, 
                                       limit: int = 100) -> List[Dict[str, Any]]:
        """
        Get rule execution history from chaincode library.
        
        Args:
            rule_id: Optional rule ID filter
            limit: Maximum number of records to return
            
        Returns:
            Rule execution history
        """
        try:
            query_request = {
                'rule_id': rule_id,
                'limit': limit,
                'gateway_id': await self._get_gateway_id()
            }
            
            # Call chaincode library policy-enforcement
            result = await self.blockchain_client.call_chaincode(
                'policy-enforcement',
                'getRuleExecutionHistory',
                query_request
            )
            
            return result.get('executions', [])
            
        except Exception as e:
            logger.error(f"Get rule execution history failed: {e}")
            return []
    
    def register_rule_handler(self, action_type: str, handler: Callable):
        """
        Register a local handler for rule actions.
        
        Args:
            action_type: Type of action to handle
            handler: Async function to handle the action
        """
        self.rule_handlers[action_type] = handler
        logger.info(f"Registered rule handler for action type: {action_type}")
    
    async def _execute_rule_actions(self, rule_results: List[Dict[str, Any]]):
        """Execute local actions for matched rules."""
        for result in rule_results:
            if result.get('matched', False):
                actions = result.get('actions', [])
                for action in actions:
                    await self._execute_action(action, result)
    
    async def _execute_action(self, action: Dict[str, Any], rule_result: Dict[str, Any]):
        """Execute a single rule action."""
        try:
            action_type = action.get('type')
            action_params = action.get('parameters', {})
            
            if action_type in self.rule_handlers:
                handler = self.rule_handlers[action_type]
                await handler(action_params, rule_result)
                logger.debug(f"Executed action {action_type}")
            else:
                logger.warning(f"No handler registered for action type: {action_type}")
                
        except Exception as e:
            logger.error(f"Action execution failed: {e}")
    
    def _register_builtin_handlers(self):
        """Register built-in rule action handlers."""
        
        async def log_action(params: Dict[str, Any], rule_result: Dict[str, Any]):
            """Built-in logging action."""
            level = params.get('level', 'info').lower()
            message = params.get('message', 'Rule action executed')
            
            if level == 'debug':
                logger.debug(message)
            elif level == 'warning':
                logger.warning(message)
            elif level == 'error':
                logger.error(message)
            else:
                logger.info(message)
        
        async def block_device_action(params: Dict[str, Any], rule_result: Dict[str, Any]):
            """Built-in device blocking action."""
            device_id = params.get('device_id')
            duration = params.get('duration', 3600)  # 1 hour default
            
            if device_id:
                # This would integrate with the access control system
                logger.warning(f"Blocking device {device_id} for {duration} seconds")
                # TODO: Implement actual device blocking
        
        async def send_alert_action(params: Dict[str, Any], rule_result: Dict[str, Any]):
            """Built-in alert sending action."""
            alert_type = params.get('alert_type', 'security')
            message = params.get('message', 'Rule triggered alert')
            recipients = params.get('recipients', [])
            
            logger.warning(f"ALERT [{alert_type}]: {message}")
            # TODO: Implement actual alert sending (email, SMS, etc.)
        
        # Register built-in handlers
        self.register_rule_handler('log', log_action)
        self.register_rule_handler('block_device', block_device_action)
        self.register_rule_handler('send_alert', send_alert_action)
    
    def _generate_evaluation_cache_key(self, event_type: str, event_data: Dict[str, Any]) -> str:
        """Generate cache key for rule evaluation."""
        # Create a deterministic hash of event type and key event data
        key_data = {
            'event_type': event_type,
            'device_id': event_data.get('device_id'),
            'resource': event_data.get('resource'),
            'action': event_data.get('action')
        }
        return json.dumps(key_data, sort_keys=True)
    
    def _is_evaluation_cache_valid(self, cache_key: str) -> bool:
        """Check if evaluation cache entry is still valid."""
        if cache_key not in self.evaluation_cache:
            return False
        
        entry = self.evaluation_cache[cache_key]
        age = (datetime.utcnow() - entry['timestamp']).total_seconds()
        return age < entry['ttl']
    
    def _clear_evaluation_cache(self):
        """Clear all evaluation cache entries."""
        self.evaluation_cache.clear()
    
    async def _get_gateway_id(self) -> str:
        """Get the gateway ID from blockchain client."""
        return getattr(self.blockchain_client, 'gateway_id', 'unknown-gateway')
    
    async def get_rule_stats(self) -> Dict[str, Any]:
        """Get rule engine statistics."""
        return {
            'cached_rules': len(self.rules_cache),
            'registered_handlers': len(self.rule_handlers),
            'evaluation_cache_size': len(self.evaluation_cache),
            'total_evaluations': getattr(self, '_total_evaluations', 0),
            'total_matches': getattr(self, '_total_matches', 0),
            'total_actions_executed': getattr(self, '_total_actions', 0)
        }
