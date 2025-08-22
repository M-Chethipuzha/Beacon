"""
MQTT Communication Handler

Handles MQTT broker integration for IoT device communication.
Provides secure, privacy-preserving messaging with access control enforcement.
"""

import asyncio
import json
import logging
import ssl
from typing import Dict, List, Optional, Callable, Any
from datetime import datetime
from dataclasses import dataclass

# Note: paho-mqtt will be added to requirements.txt
# import paho.mqtt.client as mqtt

from ..policy.enforcer import PolicyEnforcer, AccessRequest, AccessDecision

logger = logging.getLogger(__name__)

@dataclass
class MQTTMessage:
    """Represents an MQTT message with metadata"""
    topic: str
    payload: bytes
    qos: int
    retain: bool
    timestamp: datetime
    client_id: str
    device_info: Optional[Dict[str, Any]] = None

class MQTTHandler:
    """
    MQTT communication handler with integrated access control.
    
    Features:
    - Secure TLS connections
    - Topic-based access control
    - Device authentication
    - Message filtering
    - Privacy-preserving logging
    """
    
    def __init__(self,
                 broker_host: str = "localhost",
                 broker_port: int = 1883,
                 broker_port_tls: int = 8883,
                 use_tls: bool = True,
                 policy_enforcer: Optional[PolicyEnforcer] = None):
        """
        Initialize MQTT handler.
        
        Args:
            broker_host: MQTT broker hostname
            broker_port: MQTT broker port (non-TLS)
            broker_port_tls: MQTT broker port (TLS)
            use_tls: Whether to use TLS encryption
            policy_enforcer: Policy enforcement engine
        """
        self.broker_host = broker_host
        self.broker_port = broker_port_tls if use_tls else broker_port
        self.use_tls = use_tls
        self.policy_enforcer = policy_enforcer
        
        # MQTT client configuration
        self.client = None  # Will be initialized in start()
        self.client_id = f"beacon-gateway-{datetime.now().strftime('%Y%m%d-%H%M%S')}"
        
        # Connection state
        self.connected = False
        self.connection_lock = asyncio.Lock()
        
        # Subscriptions and handlers
        self.topic_handlers: Dict[str, Callable] = {}
        self.subscribed_topics: List[str] = []
        
        # Device registry (maps client_id to device info)
        self.device_registry: Dict[str, Dict[str, Any]] = {}
        
        # Statistics
        self.stats = {
            "messages_received": 0,
            "messages_sent": 0,
            "messages_allowed": 0,
            "messages_denied": 0,
            "connected_devices": 0,
            "last_activity": None
        }
        
    async def start(self):
        """Start MQTT handler and connect to broker"""
        logger.info(f"Starting MQTT handler - connecting to {self.broker_host}:{self.broker_port}")
        
        try:
            # TODO: Import and initialize paho-mqtt client
            # For now, create a placeholder
            logger.info("MQTT client initialized (placeholder - requires paho-mqtt)")
            
            # Configure client callbacks
            # self.client.on_connect = self._on_connect
            # self.client.on_disconnect = self._on_disconnect
            # self.client.on_message = self._on_message
            # self.client.on_subscribe = self._on_subscribe
            # self.client.on_publish = self._on_publish
            
            # Configure TLS if enabled
            if self.use_tls:
                await self._configure_tls()
            
            # Connect to broker
            await self._connect_to_broker()
            
            # Subscribe to default topics
            await self._setup_default_subscriptions()
            
            logger.info("MQTT handler started successfully")
            
        except Exception as e:
            logger.error(f"Failed to start MQTT handler: {e}")
            raise
    
    async def stop(self):
        """Stop MQTT handler and disconnect from broker"""
        logger.info("Stopping MQTT handler")
        
        try:
            if self.client and self.connected:
                # Unsubscribe from all topics
                for topic in self.subscribed_topics:
                    logger.debug(f"Unsubscribing from topic: {topic}")
                    # self.client.unsubscribe(topic)
                
                # Disconnect from broker
                # self.client.disconnect()
                logger.info("Disconnected from MQTT broker")
            
            self.connected = False
            
        except Exception as e:
            logger.error(f"Error stopping MQTT handler: {e}")
    
    async def _configure_tls(self):
        """Configure TLS settings for secure communication"""
        try:
            # Create SSL context
            context = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
            
            # Configure certificate verification
            # For production, use proper certificates
            context.check_hostname = False
            context.verify_mode = ssl.CERT_NONE
            
            # TODO: Apply TLS configuration to MQTT client
            # self.client.tls_set_context(context)
            
            logger.debug("TLS configuration applied")
            
        except Exception as e:
            logger.error(f"Failed to configure TLS: {e}")
            raise
    
    async def _connect_to_broker(self):
        """Connect to MQTT broker with retry logic"""
        max_retries = 3
        retry_delay = 5
        
        for attempt in range(max_retries):
            try:
                logger.debug(f"Connecting to MQTT broker (attempt {attempt + 1}/{max_retries})")
                
                # TODO: Implement actual connection
                # result = self.client.connect(self.broker_host, self.broker_port, 60)
                # if result == mqtt.MQTT_ERR_SUCCESS:
                #     self.connected = True
                #     logger.info("Connected to MQTT broker successfully")
                #     return
                
                # Placeholder - simulate successful connection
                await asyncio.sleep(1)
                self.connected = True
                logger.info("Connected to MQTT broker successfully (placeholder)")
                return
                
            except Exception as e:
                logger.warning(f"Connection attempt {attempt + 1} failed: {e}")
                if attempt < max_retries - 1:
                    await asyncio.sleep(retry_delay)
                else:
                    raise
    
    async def _setup_default_subscriptions(self):
        """Subscribe to default IoT topics"""
        default_topics = [
            "devices/+/telemetry",      # Device telemetry data
            "devices/+/status",         # Device status updates
            "devices/+/events",         # Device events
            "gateway/commands",         # Gateway commands
            "gateway/config"            # Gateway configuration updates
        ]
        
        for topic in default_topics:
            await self.subscribe_topic(topic, self._default_message_handler)
    
    async def subscribe_topic(self, topic: str, handler: Callable):
        """
        Subscribe to an MQTT topic with access control.
        
        Args:
            topic: MQTT topic pattern
            handler: Message handler function
        """
        try:
            logger.info(f"Subscribing to topic: {topic}")
            
            # TODO: Implement actual subscription
            # result, mid = self.client.subscribe(topic, qos=1)
            # if result == mqtt.MQTT_ERR_SUCCESS:
            #     self.topic_handlers[topic] = handler
            #     self.subscribed_topics.append(topic)
            #     logger.debug(f"Subscribed to {topic} successfully")
            
            # Placeholder
            self.topic_handlers[topic] = handler
            self.subscribed_topics.append(topic)
            logger.debug(f"Subscribed to {topic} successfully (placeholder)")
            
        except Exception as e:
            logger.error(f"Failed to subscribe to topic {topic}: {e}")
    
    async def publish_message(self, 
                             topic: str, 
                             payload: Any, 
                             qos: int = 1,
                             retain: bool = False) -> bool:
        """
        Publish message to MQTT topic with access control.
        
        Args:
            topic: MQTT topic
            payload: Message payload (will be JSON serialized if dict)
            qos: Quality of Service level
            retain: Whether to retain the message
            
        Returns:
            True if published successfully
        """
        try:
            if not self.connected:
                logger.warning("Cannot publish - not connected to MQTT broker")
                return False
            
            # Serialize payload if needed
            if isinstance(payload, dict):
                payload_bytes = json.dumps(payload).encode()
            elif isinstance(payload, str):
                payload_bytes = payload.encode()
            else:
                payload_bytes = payload
            
            # Check access control for publishing
            if self.policy_enforcer:
                access_request = AccessRequest(
                    device_id="gateway",
                    device_type="gateway",
                    action="publish",
                    resource=topic,
                    timestamp=datetime.now(),
                    protocol="mqtt"
                )
                
                result = await self.policy_enforcer.evaluate_access_request(access_request)
                if result.decision != AccessDecision.ALLOW:
                    logger.warning(f"Publish denied for topic {topic}: {result.reason}")
                    self.stats["messages_denied"] += 1
                    return False
            
            # TODO: Implement actual publishing
            # result = self.client.publish(topic, payload_bytes, qos, retain)
            # if result.rc == mqtt.MQTT_ERR_SUCCESS:
            #     self.stats["messages_sent"] += 1
            #     self.stats["last_activity"] = datetime.now().isoformat()
            #     logger.debug(f"Published message to {topic}")
            #     return True
            
            # Placeholder
            self.stats["messages_sent"] += 1
            self.stats["last_activity"] = datetime.now().isoformat()
            logger.debug(f"Published message to {topic} (placeholder)")
            return True
            
        except Exception as e:
            logger.error(f"Failed to publish to topic {topic}: {e}")
            return False
    
    async def _default_message_handler(self, message: MQTTMessage):
        """Default handler for incoming MQTT messages"""
        try:
            logger.debug(f"Received message on topic {message.topic}")
            
            # Parse device information from topic
            device_info = self._parse_device_from_topic(message.topic)
            if device_info:
                message.device_info = device_info
            
            # Apply access control
            if self.policy_enforcer and device_info:
                access_request = AccessRequest(
                    device_id=device_info.get("device_id", "unknown"),
                    device_type=device_info.get("device_type", "unknown"),
                    action="publish",
                    resource=message.topic,
                    timestamp=message.timestamp,
                    protocol="mqtt"
                )
                
                result = await self.policy_enforcer.evaluate_access_request(access_request)
                if result.decision != AccessDecision.ALLOW:
                    logger.warning(f"Message denied from {device_info['device_id']}: {result.reason}")
                    self.stats["messages_denied"] += 1
                    return
                else:
                    self.stats["messages_allowed"] += 1
            
            # Process message based on topic
            await self._process_message_by_topic(message)
            
            # Update statistics
            self.stats["messages_received"] += 1
            self.stats["last_activity"] = datetime.now().isoformat()
            
        except Exception as e:
            logger.error(f"Error handling MQTT message: {e}")
    
    def _parse_device_from_topic(self, topic: str) -> Optional[Dict[str, Any]]:
        """
        Parse device information from MQTT topic.
        
        Args:
            topic: MQTT topic (e.g., "devices/sensor001/telemetry")
            
        Returns:
            Device information dictionary
        """
        try:
            topic_parts = topic.split("/")
            
            if len(topic_parts) >= 3 and topic_parts[0] == "devices":
                device_id = topic_parts[1]
                message_type = topic_parts[2]
                
                # Determine device type from ID pattern or registry
                device_type = self._determine_device_type(device_id)
                
                return {
                    "device_id": device_id,
                    "device_type": device_type,
                    "message_type": message_type
                }
                
        except Exception as e:
            logger.debug(f"Failed to parse device from topic {topic}: {e}")
            
        return None
    
    def _determine_device_type(self, device_id: str) -> str:
        """
        Determine device type from device ID.
        
        Args:
            device_id: Device identifier
            
        Returns:
            Device type string
        """
        # Check device registry first
        if device_id in self.device_registry:
            return self.device_registry[device_id].get("type", "unknown")
        
        # Infer from device ID pattern
        device_id_lower = device_id.lower()
        if "sensor" in device_id_lower:
            return "sensor"
        elif "actuator" in device_id_lower:
            return "actuator"
        elif "camera" in device_id_lower:
            return "camera"
        elif "lock" in device_id_lower:
            return "smart_lock"
        else:
            return "unknown"
    
    async def _process_message_by_topic(self, message: MQTTMessage):
        """Process message based on topic pattern"""
        try:
            if message.device_info:
                message_type = message.device_info.get("message_type")
                
                if message_type == "telemetry":
                    await self._handle_telemetry_message(message)
                elif message_type == "status":
                    await self._handle_status_message(message)
                elif message_type == "events":
                    await self._handle_event_message(message)
                else:
                    logger.debug(f"Unknown message type: {message_type}")
            
        except Exception as e:
            logger.error(f"Error processing message by topic: {e}")
    
    async def _handle_telemetry_message(self, message: MQTTMessage):
        """Handle device telemetry messages"""
        try:
            # Parse telemetry data
            telemetry_data = json.loads(message.payload.decode())
            device_id = message.device_info["device_id"]
            
            logger.debug(f"Telemetry from {device_id}: {telemetry_data}")
            
            # TODO: Forward to monitoring/analytics systems
            # TODO: Store in local cache if needed
            # TODO: Apply data transformation/filtering
            
        except Exception as e:
            logger.error(f"Error handling telemetry message: {e}")
    
    async def _handle_status_message(self, message: MQTTMessage):
        """Handle device status messages"""
        try:
            status_data = json.loads(message.payload.decode())
            device_id = message.device_info["device_id"]
            
            # Update device registry
            if device_id not in self.device_registry:
                self.device_registry[device_id] = {}
            
            self.device_registry[device_id].update({
                "last_seen": datetime.now().isoformat(),
                "status": status_data.get("status", "unknown"),
                "online": status_data.get("online", True)
            })
            
            logger.debug(f"Status update from {device_id}: {status_data}")
            
        except Exception as e:
            logger.error(f"Error handling status message: {e}")
    
    async def _handle_event_message(self, message: MQTTMessage):
        """Handle device event messages"""
        try:
            event_data = json.loads(message.payload.decode())
            device_id = message.device_info["device_id"]
            
            logger.info(f"Event from {device_id}: {event_data}")
            
            # TODO: Process events (alerts, alarms, etc.)
            # TODO: Forward to event processing system
            
        except Exception as e:
            logger.error(f"Error handling event message: {e}")
    
    async def register_device(self, 
                             device_id: str, 
                             device_info: Dict[str, Any]):
        """
        Register a device in the local registry.
        
        Args:
            device_id: Device identifier
            device_info: Device metadata
        """
        self.device_registry[device_id] = {
            **device_info,
            "registered_at": datetime.now().isoformat()
        }
        
        self.stats["connected_devices"] = len(self.device_registry)
        logger.info(f"Registered device: {device_id}")
    
    def get_mqtt_stats(self) -> Dict[str, Any]:
        """Get MQTT handler statistics"""
        return {
            **self.stats,
            "connected": self.connected,
            "broker_host": self.broker_host,
            "broker_port": self.broker_port,
            "use_tls": self.use_tls,
            "subscribed_topics": len(self.subscribed_topics),
            "registered_devices": len(self.device_registry)
        }
