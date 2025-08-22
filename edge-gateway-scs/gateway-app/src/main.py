"""
BEACON Edge Gateway SCS - Main Application

Entry point for the Edge Gateway Smart Contract Service.
Integrates all components for IoT device management, policy enforcement,
and blockchain communication with the I&O SCS.
"""

import asyncio
import logging
import signal
import sys
import os
from datetime import datetime
from pathlib import Path

# Add src directory to Python path
sys.path.insert(0, str(Path(__file__).parent))

from discovery.io_scs_discovery import IOSCSDiscovery
from policy.cache import PolicyCache
from policy.enforcer import PolicyEnforcer
from iot.mqtt_handler import MQTTHandler
from blockchain.client import BlockchainClient, GatewayInfo

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler('logs/gateway.log', mode='a')
    ]
)

logger = logging.getLogger(__name__)

class EdgeGatewayService:
    """
    Main Edge Gateway Service class.
    
    Orchestrates all components of the Edge Gateway SCS:
    - I&O SCS discovery and communication
    - Policy caching and enforcement
    - IoT device communication (MQTT/CoAP)
    - VPN connectivity
    - Local API server
    """
    
    def __init__(self, config: dict = None):
        """
        Initialize Edge Gateway Service.
        
        Args:
            config: Configuration dictionary
        """
        self.config = config or self._load_default_config()
        self.running = False
        
        # Initialize components
        self.discovery_service: IOSCSDiscovery = None
        self.policy_cache: PolicyCache = None
        self.policy_enforcer: PolicyEnforcer = None
        self.mqtt_handler: MQTTHandler = None
        self.blockchain_client: BlockchainClient = None
        
        # Additional components
        self.api_server = None
        self.coap_handler = None
        
        # Setup signal handlers for graceful shutdown
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)
        
    def _load_default_config(self) -> dict:
        """Load default configuration"""
        return {
            "gateway": {
                "id": f"gateway-{datetime.now().strftime('%Y%m%d-%H%M%S')}",
                "name": "BEACON Edge Gateway",
                "location": "Default Location",
                "capabilities": ["mqtt", "coap", "policy_enforcement"]
            },
            "io_scs": {
                "discovery_domains": ["_beacon-io._tcp.beacon.local"],
                "static_nodes": ["localhost:8080"],
                "health_check_interval": 30,
                "connection_timeout": 10
            },
            "mqtt": {
                "broker_host": "localhost",
                "broker_port": 1883,
                "broker_port_tls": 8883,
                "use_tls": False
            },
            "policy": {
                "cache_file": "data/policies.db",
                "sync_interval": 300,
                "default_decision": "deny"
            },
            "logging": {
                "level": "INFO",
                "file": "logs/gateway.log"
            },
            "api": {
                "host": "0.0.0.0",
                "port": 8081,
                "enable_cors": True
            }
        }
    
    async def start(self):
        """Start all gateway services"""
        logger.info("Starting BEACON Edge Gateway Service")
        
        try:
            # Create necessary directories
            os.makedirs("data", exist_ok=True)
            os.makedirs("logs", exist_ok=True)
            
            # Initialize and start I&O SCS discovery
            await self._start_discovery_service()
            
            # Initialize policy cache and enforcer
            await self._start_policy_services()
            
            # Initialize IoT communication handlers
            await self._start_iot_services()
            
            # Initialize blockchain client
            await self._start_blockchain_client()
            
            # Start local API server
            await self._start_api_server()
            
            # Start CoAP server
            await self._start_coap_server()
            
            self.running = True
            logger.info("BEACON Edge Gateway Service started successfully")
            logger.info(f"Gateway ID: {self.config['gateway']['id']}")
            
            # Keep the service running
            await self._run_forever()
            
        except Exception as e:
            logger.error(f"Failed to start Edge Gateway Service: {e}")
            await self.stop()
            raise
    
    async def stop(self):
        """Stop all gateway services"""
        logger.info("Stopping BEACON Edge Gateway Service")
        
        self.running = False
        
        # Stop services in reverse order
        if self.blockchain_client:
            await self.blockchain_client.stop()
        
        if self.coap_handler:
            await self.coap_handler.stop()
        
        if self.api_server:
            await self.api_server.stop()
        
        if self.mqtt_handler:
            await self.mqtt_handler.stop()
        
        if self.policy_cache:
            await self.policy_cache.close()
        
        if self.discovery_service:
            await self.discovery_service.stop()
        
        logger.info("BEACON Edge Gateway Service stopped")
    
    async def _start_discovery_service(self):
        """Initialize and start I&O SCS discovery service"""
        logger.info("Starting I&O SCS discovery service")
        
        self.discovery_service = IOSCSDiscovery(
            discovery_domains=self.config["io_scs"]["discovery_domains"],
            static_nodes=self.config["io_scs"]["static_nodes"],
            health_check_interval=self.config["io_scs"]["health_check_interval"],
            connection_timeout=self.config["io_scs"]["connection_timeout"]
        )
        
        await self.discovery_service.start()
        logger.info("I&O SCS discovery service started")
    
    async def _start_policy_services(self):
        """Initialize policy cache and enforcement engine"""
        logger.info("Starting policy services")
        
        # Initialize policy cache
        self.policy_cache = PolicyCache(
            cache_file=self.config["policy"]["cache_file"]
        )
        await self.policy_cache.initialize()
        
        # Initialize policy enforcer
        default_decision_str = self.config["policy"]["default_decision"]
        from policy.enforcer import AccessDecision
        default_decision = AccessDecision.DENY if default_decision_str == "deny" else AccessDecision.ALLOW
        
        self.policy_enforcer = PolicyEnforcer(
            policy_cache=self.policy_cache,
            gateway_id=self.config["gateway"]["id"],
            default_decision=default_decision
        )
        
        logger.info("Policy services started")
    
    async def _start_iot_services(self):
        """Initialize IoT communication services"""
        logger.info("Starting IoT communication services")
        
        # Initialize MQTT handler
        self.mqtt_handler = MQTTHandler(
            broker_host=self.config["mqtt"]["broker_host"],
            broker_port=self.config["mqtt"]["broker_port"],
            broker_port_tls=self.config["mqtt"]["broker_port_tls"],
            use_tls=self.config["mqtt"]["use_tls"],
            policy_enforcer=self.policy_enforcer
        )
        
        await self.mqtt_handler.start()
        
        # TODO: Initialize CoAP handler
        # TODO: Initialize VPN client
        
        logger.info("IoT communication services started")
    
    async def _start_blockchain_client(self):
        """Initialize blockchain client for I&O SCS communication"""
        logger.info("Starting blockchain client")
        
        # Prepare gateway information
        gateway_info = GatewayInfo(
            gateway_id=self.config["gateway"]["id"],
            name=self.config["gateway"]["name"],
            location=self.config["gateway"]["location"],
            capabilities=self.config["gateway"]["capabilities"],
            public_key="placeholder_public_key",  # TODO: Generate actual key
            network_info={
                "internal_ip": "192.168.1.100",  # TODO: Get actual IP
                "external_ip": "auto-detect",
                "mqtt_port": self.config["mqtt"]["broker_port"]
            },
            metadata={
                "version": "1.0.0",
                "started_at": datetime.now().isoformat()
            }
        )
        
        self.blockchain_client = BlockchainClient(
            discovery_service=self.discovery_service,
            gateway_info=gateway_info,
            sync_interval=self.config["policy"]["sync_interval"]
        )
        
        await self.blockchain_client.start()
        logger.info("Blockchain client started")
    
    async def _run_forever(self):
        """Keep the service running until stopped"""
        try:
            while self.running:
                # Perform periodic health checks
                await self._health_check()
                
                # Sleep for a short interval
                await asyncio.sleep(10)
                
        except asyncio.CancelledError:
            logger.info("Service execution cancelled")
        except Exception as e:
            logger.error(f"Error in main service loop: {e}")
            raise
    
    async def _health_check(self):
        """Perform periodic health checks on all services"""
        try:
            # Check I&O SCS connectivity
            discovery_status = self.discovery_service.get_status()
            if discovery_status["healthy_nodes"] == 0:
                logger.warning("No healthy I&O SCS nodes available")
            
            # Check policy cache status
            cache_stats = await self.policy_cache.get_cache_stats()
            if cache_stats.get("enabled_policies", 0) == 0:
                logger.warning("No policies available in cache")
            
            # Check MQTT connectivity
            mqtt_stats = self.mqtt_handler.get_mqtt_stats()
            if not mqtt_stats["connected"]:
                logger.warning("MQTT broker not connected")
            
            # Check blockchain client status
            blockchain_stats = self.blockchain_client.get_blockchain_stats()
            if not blockchain_stats["registered"]:
                logger.warning("Gateway not registered with I&O SCS")
            
        except Exception as e:
            logger.debug(f"Health check error: {e}")
    
    async def _start_api_server(self):
        """Start local API server"""
        logger.info("Starting local API server")
        
        try:
            from .api.server import APIServer
            self.api_server = APIServer(self)
            
            api_config = self.config.get('api', {})
            host = api_config.get('host', '0.0.0.0')
            port = api_config.get('port', 8081)
            
            await self.api_server.start(host, port)
            
        except Exception as e:
            logger.error(f"Failed to start API server: {e}")
    
    async def _start_coap_server(self):
        """Start CoAP server"""
        logger.info("Starting CoAP server")
        
        try:
            from .iot.coap_handler import CoAPHandler
            self.coap_handler = CoAPHandler(self)
            await self.coap_handler.start()
            
        except Exception as e:
            logger.error(f"Failed to start CoAP server: {e}")

    def _signal_handler(self, signum, frame):
        """Handle shutdown signals"""
        logger.info(f"Received signal {signum}, initiating shutdown")
        self.running = False
    
    def get_service_status(self) -> dict:
        """Get comprehensive service status"""
        try:
            return {
                "running": self.running,
                "gateway_id": self.config["gateway"]["id"],
                "discovery": self.discovery_service.get_status() if self.discovery_service else None,
                "policy_cache": asyncio.create_task(self.policy_cache.get_cache_stats()) if self.policy_cache else None,
                "policy_enforcer": self.policy_enforcer.get_enforcement_stats() if self.policy_enforcer else None,
                "mqtt": self.mqtt_handler.get_mqtt_stats() if self.mqtt_handler else None,
                "blockchain": self.blockchain_client.get_blockchain_stats() if self.blockchain_client else None
            }
        except Exception as e:
            logger.error(f"Error getting service status: {e}")
            return {"error": str(e)}

async def main():
    """Main entry point"""
    logger.info("BEACON Edge Gateway SCS starting...")
    
    try:
        # Create and start the gateway service
        gateway_service = EdgeGatewayService()
        await gateway_service.start()
        
    except KeyboardInterrupt:
        logger.info("Shutdown requested by user")
    except Exception as e:
        logger.error(f"Service error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    # Run the main service
    asyncio.run(main())
