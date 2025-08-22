"""
BEACON Edge Gateway - VPN Client Manager
=======================================

This module provides VPN tunnel management for secure communication
between edge gateways and the central BEACON infrastructure.
"""

import asyncio
import logging
import subprocess
import os
import signal
from typing import Dict, Any, Optional, List
from datetime import datetime
from enum import Enum

logger = logging.getLogger(__name__)


class VPNStatus(Enum):
    """VPN connection status."""
    DISCONNECTED = "disconnected"
    CONNECTING = "connecting"
    CONNECTED = "connected"
    RECONNECTING = "reconnecting"
    ERROR = "error"


class VPNType(Enum):
    """Supported VPN types."""
    OPENVPN = "openvpn"
    WIREGUARD = "wireguard"


class VPNClient:
    """VPN client manager for secure gateway communication."""
    
    def __init__(self, config: Dict[str, Any]):
        """Initialize VPN client with configuration."""
        self.config = config
        self.vpn_type = VPNType(config.get('type', 'openvpn'))
        self.enabled = config.get('enabled', False)
        self.config_file = config.get('config_file', '')
        self.auto_reconnect = config.get('auto_reconnect', True)
        self.connection_timeout = config.get('connection_timeout', 30)
        
        self.status = VPNStatus.DISCONNECTED
        self.process = None
        self.last_connected = None
        self.connection_attempts = 0
        self.max_reconnect_attempts = 5
        
        # Start connection manager if enabled
        if self.enabled:
            asyncio.create_task(self._connection_manager())
    
    async def connect(self) -> bool:
        """
        Establish VPN connection.
        
        Returns:
            Connection success status
        """
        if not self.enabled:
            logger.warning("VPN client is disabled")
            return False
        
        if self.status == VPNStatus.CONNECTED:
            logger.info("VPN already connected")
            return True
        
        try:
            logger.info(f"Connecting to VPN using {self.vpn_type.value}")
            self.status = VPNStatus.CONNECTING
            
            if self.vpn_type == VPNType.OPENVPN:
                success = await self._connect_openvpn()
            elif self.vpn_type == VPNType.WIREGUARD:
                success = await self._connect_wireguard()
            else:
                logger.error(f"Unsupported VPN type: {self.vpn_type}")
                success = False
            
            if success:
                self.status = VPNStatus.CONNECTED
                self.last_connected = datetime.utcnow()
                self.connection_attempts = 0
                logger.info("VPN connection established successfully")
            else:
                self.status = VPNStatus.ERROR
                self.connection_attempts += 1
                logger.error("VPN connection failed")
            
            return success
            
        except Exception as e:
            logger.error(f"VPN connection error: {e}")
            self.status = VPNStatus.ERROR
            return False
    
    async def disconnect(self) -> bool:
        """
        Disconnect VPN connection.
        
        Returns:
            Disconnection success status
        """
        try:
            if self.status == VPNStatus.DISCONNECTED:
                logger.info("VPN already disconnected")
                return True
            
            logger.info("Disconnecting VPN")
            
            if self.process:
                # Gracefully terminate the VPN process
                if self.vpn_type == VPNType.OPENVPN:
                    await self._disconnect_openvpn()
                elif self.vpn_type == VPNType.WIREGUARD:
                    await self._disconnect_wireguard()
                
                # Wait for process to terminate
                try:
                    await asyncio.wait_for(self._wait_for_process_termination(), timeout=10)
                except asyncio.TimeoutError:
                    logger.warning("VPN process did not terminate gracefully, forcing kill")
                    self._force_kill_process()
            
            self.status = VPNStatus.DISCONNECTED
            self.process = None
            logger.info("VPN disconnected successfully")
            return True
            
        except Exception as e:
            logger.error(f"VPN disconnection error: {e}")
            return False
    
    async def reconnect(self) -> bool:
        """
        Reconnect VPN connection.
        
        Returns:
            Reconnection success status
        """
        logger.info("Reconnecting VPN")
        self.status = VPNStatus.RECONNECTING
        
        # Disconnect first
        await self.disconnect()
        
        # Wait a bit before reconnecting
        await asyncio.sleep(5)
        
        # Reconnect
        return await self.connect()
    
    async def get_status(self) -> Dict[str, Any]:
        """
        Get VPN connection status.
        
        Returns:
            VPN status information
        """
        try:
            connection_info = {}
            
            if self.status == VPNStatus.CONNECTED:
                connection_info = await self._get_connection_info()
            
            return {
                'enabled': self.enabled,
                'status': self.status.value,
                'vpn_type': self.vpn_type.value,
                'last_connected': self.last_connected.isoformat() if self.last_connected else None,
                'connection_attempts': self.connection_attempts,
                'auto_reconnect': self.auto_reconnect,
                'config_file': self.config_file,
                'connection_info': connection_info
            }
            
        except Exception as e:
            logger.error(f"Get VPN status failed: {e}")
            return {'status': 'error', 'error': str(e)}
    
    async def test_connection(self) -> Dict[str, Any]:
        """
        Test VPN connection quality.
        
        Returns:
            Connection test results
        """
        if self.status != VPNStatus.CONNECTED:
            return {'connected': False, 'error': 'VPN not connected'}
        
        try:
            # Test connectivity to various endpoints
            test_results = {
                'connected': True,
                'tests': {}
            }
            
            # Test DNS resolution
            test_results['tests']['dns'] = await self._test_dns()
            
            # Test gateway connectivity
            test_results['tests']['gateway'] = await self._test_gateway_connectivity()
            
            # Test bandwidth (if configured)
            if self.config.get('test_bandwidth', False):
                test_results['tests']['bandwidth'] = await self._test_bandwidth()
            
            return test_results
            
        except Exception as e:
            logger.error(f"VPN connection test failed: {e}")
            return {'connected': False, 'error': str(e)}
    
    async def _connect_openvpn(self) -> bool:
        """Connect using OpenVPN."""
        try:
            if not os.path.exists(self.config_file):
                logger.error(f"OpenVPN config file not found: {self.config_file}")
                return False
            
            # Build OpenVPN command
            cmd = [
                'openvpn',
                '--config', self.config_file,
                '--daemon',
                '--writepid', '/tmp/beacon-openvpn.pid',
                '--log', '/tmp/beacon-openvpn.log',
                '--verb', '3'
            ]
            
            # Add authentication if configured
            auth_file = self.config.get('auth_file')
            if auth_file and os.path.exists(auth_file):
                cmd.extend(['--auth-user-pass', auth_file])
            
            # Start OpenVPN process
            self.process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            # Wait for connection establishment
            return await self._wait_for_openvpn_connection()
            
        except Exception as e:
            logger.error(f"OpenVPN connection failed: {e}")
            return False
    
    async def _connect_wireguard(self) -> bool:
        """Connect using WireGuard."""
        try:
            if not os.path.exists(self.config_file):
                logger.error(f"WireGuard config file not found: {self.config_file}")
                return False
            
            # Use wg-quick to bring up the interface
            interface_name = self.config.get('interface', 'wg0')
            
            cmd = ['wg-quick', 'up', self.config_file]
            
            result = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            stdout, stderr = await result.communicate()
            
            if result.returncode == 0:
                logger.info("WireGuard interface brought up successfully")
                return True
            else:
                logger.error(f"WireGuard connection failed: {stderr.decode()}")
                return False
            
        except Exception as e:
            logger.error(f"WireGuard connection failed: {e}")
            return False
    
    async def _disconnect_openvpn(self):
        """Disconnect OpenVPN."""
        try:
            # Send SIGTERM to OpenVPN process
            if self.process:
                self.process.terminate()
                
            # Also try to kill by PID file
            pid_file = '/tmp/beacon-openvpn.pid'
            if os.path.exists(pid_file):
                with open(pid_file, 'r') as f:
                    pid = int(f.read().strip())
                    os.kill(pid, signal.SIGTERM)
                os.remove(pid_file)
                
        except Exception as e:
            logger.error(f"OpenVPN disconnect error: {e}")
    
    async def _disconnect_wireguard(self):
        """Disconnect WireGuard."""
        try:
            cmd = ['wg-quick', 'down', self.config_file]
            
            result = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            await result.communicate()
            
        except Exception as e:
            logger.error(f"WireGuard disconnect error: {e}")
    
    async def _wait_for_openvpn_connection(self) -> bool:
        """Wait for OpenVPN connection to establish."""
        for _ in range(self.connection_timeout):
            await asyncio.sleep(1)
            
            # Check log file for connection status
            log_file = '/tmp/beacon-openvpn.log'
            if os.path.exists(log_file):
                with open(log_file, 'r') as f:
                    log_content = f.read()
                    if 'Initialization Sequence Completed' in log_content:
                        return True
                    if 'FATAL' in log_content or 'ERROR' in log_content:
                        return False
        
        return False
    
    async def _wait_for_process_termination(self):
        """Wait for VPN process to terminate."""
        if self.process:
            await self.process.wait()
    
    def _force_kill_process(self):
        """Force kill VPN process."""
        if self.process:
            try:
                self.process.kill()
            except Exception as e:
                logger.error(f"Force kill process error: {e}")
    
    async def _get_connection_info(self) -> Dict[str, Any]:
        """Get detailed connection information."""
        try:
            info = {}
            
            if self.vpn_type == VPNType.OPENVPN:
                # Get OpenVPN status
                log_file = '/tmp/beacon-openvpn.log'
                if os.path.exists(log_file):
                    with open(log_file, 'r') as f:
                        log_content = f.read()
                        # Parse relevant information from log
                        info['log_entries'] = log_content.split('\n')[-10:]  # Last 10 lines
            
            elif self.vpn_type == VPNType.WIREGUARD:
                # Get WireGuard status
                cmd = ['wg', 'show']
                result = await asyncio.create_subprocess_exec(
                    *cmd,
                    stdout=asyncio.subprocess.PIPE,
                    stderr=asyncio.subprocess.PIPE
                )
                stdout, stderr = await result.communicate()
                if result.returncode == 0:
                    info['wg_status'] = stdout.decode()
            
            return info
            
        except Exception as e:
            logger.error(f"Get connection info failed: {e}")
            return {}
    
    async def _test_dns(self) -> Dict[str, Any]:
        """Test DNS resolution through VPN."""
        try:
            import socket
            
            test_domains = ['google.com', 'cloudflare.com']
            results = {}
            
            for domain in test_domains:
                try:
                    start_time = datetime.utcnow()
                    socket.gethostbyname(domain)
                    end_time = datetime.utcnow()
                    
                    results[domain] = {
                        'success': True,
                        'response_time': (end_time - start_time).total_seconds()
                    }
                except Exception as e:
                    results[domain] = {
                        'success': False,
                        'error': str(e)
                    }
            
            return results
            
        except Exception as e:
            return {'error': str(e)}
    
    async def _test_gateway_connectivity(self) -> Dict[str, Any]:
        """Test connectivity to I&O SCS gateways."""
        try:
            # This would test connectivity to known I&O SCS endpoints
            test_endpoints = self.config.get('test_endpoints', [])
            results = {}
            
            for endpoint in test_endpoints:
                try:
                    # Simple TCP connection test
                    reader, writer = await asyncio.wait_for(
                        asyncio.open_connection(endpoint['host'], endpoint['port']),
                        timeout=5
                    )
                    writer.close()
                    await writer.wait_closed()
                    
                    results[f"{endpoint['host']}:{endpoint['port']}"] = {
                        'success': True,
                        'reachable': True
                    }
                except Exception as e:
                    results[f"{endpoint['host']}:{endpoint['port']}"] = {
                        'success': False,
                        'error': str(e)
                    }
            
            return results
            
        except Exception as e:
            return {'error': str(e)}
    
    async def _test_bandwidth(self) -> Dict[str, Any]:
        """Test VPN bandwidth."""
        try:
            # Simple bandwidth test (placeholder)
            # In production, this could download/upload test data
            return {
                'upload_mbps': 10.5,
                'download_mbps': 25.3,
                'latency_ms': 45
            }
            
        except Exception as e:
            return {'error': str(e)}
    
    async def _connection_manager(self):
        """Background connection manager for auto-reconnect."""
        while True:
            try:
                if self.enabled and self.auto_reconnect:
                    if self.status in [VPNStatus.DISCONNECTED, VPNStatus.ERROR]:
                        if self.connection_attempts < self.max_reconnect_attempts:
                            logger.info("Attempting VPN auto-reconnect")
                            await self.connect()
                        else:
                            logger.error("Max reconnection attempts reached, giving up")
                            await asyncio.sleep(300)  # Wait 5 minutes before trying again
                            self.connection_attempts = 0
                
                await asyncio.sleep(30)  # Check every 30 seconds
                
            except Exception as e:
                logger.error(f"Connection manager error: {e}")
                await asyncio.sleep(60)
    
    def is_connected(self) -> bool:
        """Check if VPN is currently connected."""
        return self.status == VPNStatus.CONNECTED
