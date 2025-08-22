"""
I&O SCS Discovery Module

Handles automatic discovery and connection to Identity & Onboarding SCS blockchain nodes.
Maintains connection pool and handles failover between multiple I&O SCS endpoints.
"""

import asyncio
import aiohttp
import logging
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime, timedelta

logger = logging.getLogger(__name__)

@dataclass
class IOSCSNode:
    """Represents an I&O SCS blockchain node"""
    host: str
    port: int
    protocol: str = "http"
    api_version: str = "v1"
    last_seen: Optional[datetime] = None
    is_healthy: bool = False
    response_time_ms: float = 0.0
    
    @property
    def base_url(self) -> str:
        return f"{self.protocol}://{self.host}:{self.port}/api/{self.api_version}"
    
    @property
    def health_url(self) -> str:
        return f"{self.base_url}/health"

class IOSCSDiscovery:
    """
    Discovers and manages connections to I&O SCS blockchain nodes.
    
    Features:
    - DNS-based discovery
    - Health monitoring
    - Load balancing
    - Automatic failover
    """
    
    def __init__(self, 
                 discovery_domains: List[str] = None,
                 static_nodes: List[str] = None,
                 health_check_interval: int = 30,
                 connection_timeout: int = 10):
        """
        Initialize I&O SCS discovery service.
        
        Args:
            discovery_domains: DNS domains to query for SRV records
            static_nodes: Static list of I&O SCS nodes (host:port)
            health_check_interval: Seconds between health checks
            connection_timeout: HTTP connection timeout in seconds
        """
        self.discovery_domains = discovery_domains or ["_beacon-io._tcp.beacon.local"]
        self.static_nodes = static_nodes or ["localhost:8080"]
        self.health_check_interval = health_check_interval
        self.connection_timeout = connection_timeout
        
        self.nodes: Dict[str, IOSCSNode] = {}
        self.healthy_nodes: List[IOSCSNode] = []
        self.current_node_index = 0
        self.session: Optional[aiohttp.ClientSession] = None
        self._health_check_task: Optional[asyncio.Task] = None
        self._discovery_task: Optional[asyncio.Task] = None
        
    async def start(self):
        """Start the discovery service"""
        logger.info("Starting I&O SCS discovery service")
        
        # Create HTTP session with proper timeouts
        connector = aiohttp.TCPConnector(
            limit=10,
            limit_per_host=5,
            ttl_dns_cache=300,
            use_dns_cache=True
        )
        
        timeout = aiohttp.ClientTimeout(total=self.connection_timeout)
        self.session = aiohttp.ClientSession(
            connector=connector,
            timeout=timeout,
            headers={"User-Agent": "BEACON-Edge-Gateway/1.0"}
        )
        
        # Initialize with static nodes
        await self._add_static_nodes()
        
        # Start background tasks
        self._health_check_task = asyncio.create_task(self._health_check_loop())
        self._discovery_task = asyncio.create_task(self._discovery_loop())
        
        # Initial health check
        await self._check_all_nodes_health()
        
        logger.info(f"Discovery service started with {len(self.healthy_nodes)} healthy nodes")
    
    async def stop(self):
        """Stop the discovery service"""
        logger.info("Stopping I&O SCS discovery service")
        
        # Cancel background tasks
        if self._health_check_task:
            self._health_check_task.cancel()
            try:
                await self._health_check_task
            except asyncio.CancelledError:
                pass
                
        if self._discovery_task:
            self._discovery_task.cancel()
            try:
                await self._discovery_task
            except asyncio.CancelledError:
                pass
        
        # Close HTTP session
        if self.session:
            await self.session.close()
            
        logger.info("Discovery service stopped")
    
    async def get_healthy_node(self) -> Optional[IOSCSNode]:
        """Get next healthy I&O SCS node using round-robin"""
        if not self.healthy_nodes:
            logger.warning("No healthy I&O SCS nodes available")
            return None
            
        node = self.healthy_nodes[self.current_node_index]
        self.current_node_index = (self.current_node_index + 1) % len(self.healthy_nodes)
        
        return node
    
    async def make_request(self, 
                          endpoint: str, 
                          method: str = "GET", 
                          data: dict = None,
                          retry_count: int = 3) -> Optional[dict]:
        """
        Make HTTP request to I&O SCS with automatic failover.
        
        Args:
            endpoint: API endpoint (e.g., "/gateways/register")
            method: HTTP method
            data: Request payload
            retry_count: Number of retry attempts
            
        Returns:
            Response data or None if all nodes failed
        """
        for attempt in range(retry_count):
            node = await self.get_healthy_node()
            if not node:
                logger.error("No healthy I&O SCS nodes available for request")
                await asyncio.sleep(1)  # Brief delay before retry
                continue
                
            try:
                url = f"{node.base_url}{endpoint}"
                logger.debug(f"Making {method} request to {url}")
                
                async with self.session.request(method, url, json=data) as response:
                    if response.status == 200:
                        result = await response.json()
                        logger.debug(f"Successful request to {node.host}:{node.port}")
                        return result
                    else:
                        logger.warning(f"HTTP {response.status} from {node.host}:{node.port}")
                        
            except Exception as e:
                logger.warning(f"Request failed to {node.host}:{node.port}: {e}")
                # Mark node as unhealthy and remove from healthy list
                node.is_healthy = False
                if node in self.healthy_nodes:
                    self.healthy_nodes.remove(node)
                    
            # Brief delay before next attempt
            await asyncio.sleep(0.5)
            
        logger.error(f"All {retry_count} attempts failed for {method} {endpoint}")
        return None
    
    async def _add_static_nodes(self):
        """Add static nodes to the discovery pool"""
        for node_str in self.static_nodes:
            try:
                if "://" in node_str:
                    # Parse full URL
                    parts = node_str.split("://")
                    protocol = parts[0]
                    host_port = parts[1]
                else:
                    # Default to HTTP
                    protocol = "http"
                    host_port = node_str
                    
                if ":" in host_port:
                    host, port = host_port.rsplit(":", 1)
                    port = int(port)
                else:
                    host = host_port
                    port = 8080  # Default blockchain API port
                    
                node_id = f"{host}:{port}"
                if node_id not in self.nodes:
                    self.nodes[node_id] = IOSCSNode(
                        host=host,
                        port=port,
                        protocol=protocol
                    )
                    logger.info(f"Added static I&O SCS node: {node_id}")
                    
            except Exception as e:
                logger.error(f"Failed to parse static node '{node_str}': {e}")
    
    async def _discover_dns_nodes(self):
        """Discover I&O SCS nodes via DNS SRV records"""
        # TODO: Implement DNS SRV record discovery
        # This would query _beacon-io._tcp.beacon.local for SRV records
        logger.debug("DNS-based discovery not yet implemented")
    
    async def _check_node_health(self, node: IOSCSNode) -> bool:
        """Check health of a single I&O SCS node"""
        try:
            start_time = datetime.now()
            
            async with self.session.get(node.health_url) as response:
                response_time = (datetime.now() - start_time).total_seconds() * 1000
                
                if response.status == 200:
                    health_data = await response.json()
                    
                    # Update node status
                    node.last_seen = datetime.now()
                    node.response_time_ms = response_time
                    node.is_healthy = True
                    
                    logger.debug(f"Node {node.host}:{node.port} healthy ({response_time:.1f}ms)")
                    return True
                else:
                    logger.warning(f"Node {node.host}:{node.port} returned HTTP {response.status}")
                    
        except Exception as e:
            logger.debug(f"Health check failed for {node.host}:{node.port}: {e}")
            
        # Mark as unhealthy
        node.is_healthy = False
        return False
    
    async def _check_all_nodes_health(self):
        """Check health of all known nodes"""
        if not self.nodes:
            return
            
        logger.debug(f"Checking health of {len(self.nodes)} I&O SCS nodes")
        
        # Run health checks concurrently
        health_tasks = [
            self._check_node_health(node) 
            for node in self.nodes.values()
        ]
        
        results = await asyncio.gather(*health_tasks, return_exceptions=True)
        
        # Update healthy nodes list
        previous_healthy_count = len(self.healthy_nodes)
        self.healthy_nodes = [
            node for node in self.nodes.values() 
            if node.is_healthy
        ]
        
        # Sort by response time for better load balancing
        self.healthy_nodes.sort(key=lambda n: n.response_time_ms)
        
        current_healthy_count = len(self.healthy_nodes)
        
        if current_healthy_count != previous_healthy_count:
            logger.info(f"Healthy I&O SCS nodes: {current_healthy_count}/{len(self.nodes)}")
            
        if current_healthy_count == 0:
            logger.error("No healthy I&O SCS nodes available!")
    
    async def _health_check_loop(self):
        """Background task for periodic health checks"""
        while True:
            try:
                await asyncio.sleep(self.health_check_interval)
                await self._check_all_nodes_health()
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Health check loop error: {e}")
                await asyncio.sleep(5)  # Brief delay before retry
    
    async def _discovery_loop(self):
        """Background task for periodic node discovery"""
        while True:
            try:
                await asyncio.sleep(300)  # 5 minutes
                await self._discover_dns_nodes()
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Discovery loop error: {e}")
                await asyncio.sleep(30)  # Brief delay before retry
    
    def get_status(self) -> Dict:
        """Get current discovery service status"""
        return {
            "total_nodes": len(self.nodes),
            "healthy_nodes": len(self.healthy_nodes),
            "current_node": (
                f"{self.healthy_nodes[self.current_node_index].host}:"
                f"{self.healthy_nodes[self.current_node_index].port}"
                if self.healthy_nodes else None
            ),
            "nodes": [
                {
                    "host": node.host,
                    "port": node.port,
                    "healthy": node.is_healthy,
                    "response_time_ms": node.response_time_ms,
                    "last_seen": node.last_seen.isoformat() if node.last_seen else None
                }
                for node in self.nodes.values()
            ]
        }
