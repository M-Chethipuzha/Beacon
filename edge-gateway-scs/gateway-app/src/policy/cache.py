"""
Policy Cache Management

Handles local caching of policies from I&O SCS for offline enforcement capability.
Maintains policy synchronization and provides fallback mechanisms.
"""

import asyncio
import json
import sqlite3
import logging
from typing import Dict, List, Optional, Any
from datetime import datetime, timedelta
from dataclasses import dataclass, asdict
from pathlib import Path

logger = logging.getLogger(__name__)

@dataclass
class Policy:
    """Represents a single access control policy"""
    id: str
    name: str
    description: str
    rules: Dict[str, Any]
    priority: int
    enabled: bool
    created_at: datetime
    updated_at: datetime
    expires_at: Optional[datetime] = None
    version: int = 1
    
    def to_dict(self) -> Dict:
        """Convert policy to dictionary for storage"""
        data = asdict(self)
        # Convert datetime objects to ISO strings
        data['created_at'] = self.created_at.isoformat()
        data['updated_at'] = self.updated_at.isoformat()
        if self.expires_at:
            data['expires_at'] = self.expires_at.isoformat()
        return data
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'Policy':
        """Create policy from dictionary"""
        # Convert ISO strings back to datetime objects
        data['created_at'] = datetime.fromisoformat(data['created_at'])
        data['updated_at'] = datetime.fromisoformat(data['updated_at'])
        if data.get('expires_at'):
            data['expires_at'] = datetime.fromisoformat(data['expires_at'])
        return cls(**data)
    
    def is_expired(self) -> bool:
        """Check if policy has expired"""
        if not self.expires_at:
            return False
        return datetime.now() > self.expires_at

class PolicyCache:
    """
    Local policy cache with SQLite backend.
    
    Features:
    - Persistent storage across restarts
    - Policy versioning
    - Expiration handling
    - Synchronization tracking
    """
    
    def __init__(self, cache_file: str = "data/policies.db"):
        """
        Initialize policy cache.
        
        Args:
            cache_file: Path to SQLite database file
        """
        self.cache_file = Path(cache_file)
        self.cache_file.parent.mkdir(parents=True, exist_ok=True)
        
        self._conn: Optional[sqlite3.Connection] = None
        self._last_sync: Optional[datetime] = None
        self._sync_lock = asyncio.Lock()
        
    async def initialize(self):
        """Initialize the policy cache database"""
        logger.info(f"Initializing policy cache: {self.cache_file}")
        
        self._conn = sqlite3.connect(
            str(self.cache_file),
            check_same_thread=False,
            isolation_level=None  # Autocommit mode
        )
        
        # Enable WAL mode for better concurrency
        self._conn.execute("PRAGMA journal_mode=WAL")
        self._conn.execute("PRAGMA synchronous=NORMAL")
        
        # Create policies table
        self._conn.execute("""
            CREATE TABLE IF NOT EXISTS policies (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                rules TEXT NOT NULL,  -- JSON string
                priority INTEGER DEFAULT 0,
                enabled BOOLEAN DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                expires_at TEXT,
                version INTEGER DEFAULT 1,
                checksum TEXT,
                sync_status TEXT DEFAULT 'synced'
            )
        """)
        
        # Create sync metadata table
        self._conn.execute("""
            CREATE TABLE IF NOT EXISTS sync_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        """)
        
        # Create indexes for performance
        self._conn.execute("CREATE INDEX IF NOT EXISTS idx_policy_priority ON policies(priority DESC)")
        self._conn.execute("CREATE INDEX IF NOT EXISTS idx_policy_enabled ON policies(enabled)")
        self._conn.execute("CREATE INDEX IF NOT EXISTS idx_policy_expires ON policies(expires_at)")
        
        self._conn.commit()
        
        # Load last sync time
        cursor = self._conn.execute(
            "SELECT value FROM sync_metadata WHERE key = 'last_sync'"
        )
        row = cursor.fetchone()
        if row:
            self._last_sync = datetime.fromisoformat(row[0])
            
        logger.info("Policy cache initialized successfully")
    
    async def close(self):
        """Close the policy cache"""
        if self._conn:
            self._conn.close()
            self._conn = None
            logger.info("Policy cache closed")
    
    async def store_policy(self, policy: Policy) -> bool:
        """
        Store or update a policy in the cache.
        
        Args:
            policy: Policy object to store
            
        Returns:
            True if stored successfully
        """
        try:
            # Calculate checksum for change detection
            policy_data = policy.to_dict()
            checksum = hash(json.dumps(policy_data, sort_keys=True))
            
            self._conn.execute("""
                INSERT OR REPLACE INTO policies (
                    id, name, description, rules, priority, enabled,
                    created_at, updated_at, expires_at, version, checksum
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """, (
                policy.id,
                policy.name,
                policy.description,
                json.dumps(policy.rules),
                policy.priority,
                policy.enabled,
                policy.created_at.isoformat(),
                policy.updated_at.isoformat(),
                policy.expires_at.isoformat() if policy.expires_at else None,
                policy.version,
                str(checksum)
            ))
            
            logger.debug(f"Stored policy: {policy.id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to store policy {policy.id}: {e}")
            return False
    
    async def get_policy(self, policy_id: str) -> Optional[Policy]:
        """
        Retrieve a policy by ID.
        
        Args:
            policy_id: Policy identifier
            
        Returns:
            Policy object or None if not found
        """
        try:
            cursor = self._conn.execute("""
                SELECT id, name, description, rules, priority, enabled,
                       created_at, updated_at, expires_at, version
                FROM policies 
                WHERE id = ? AND enabled = 1
            """, (policy_id,))
            
            row = cursor.fetchone()
            if not row:
                return None
                
            return Policy(
                id=row[0],
                name=row[1],
                description=row[2],
                rules=json.loads(row[3]),
                priority=row[4],
                enabled=bool(row[5]),
                created_at=datetime.fromisoformat(row[6]),
                updated_at=datetime.fromisoformat(row[7]),
                expires_at=datetime.fromisoformat(row[8]) if row[8] else None,
                version=row[9]
            )
            
        except Exception as e:
            logger.error(f"Failed to get policy {policy_id}: {e}")
            return None
    
    async def get_all_policies(self, 
                              enabled_only: bool = True,
                              exclude_expired: bool = True) -> List[Policy]:
        """
        Retrieve all policies from cache.
        
        Args:
            enabled_only: Only return enabled policies
            exclude_expired: Exclude expired policies
            
        Returns:
            List of Policy objects
        """
        try:
            query = """
                SELECT id, name, description, rules, priority, enabled,
                       created_at, updated_at, expires_at, version
                FROM policies
            """
            conditions = []
            params = []
            
            if enabled_only:
                conditions.append("enabled = 1")
                
            if exclude_expired:
                conditions.append("(expires_at IS NULL OR expires_at > ?)")
                params.append(datetime.now().isoformat())
                
            if conditions:
                query += " WHERE " + " AND ".join(conditions)
                
            query += " ORDER BY priority DESC"
            
            cursor = self._conn.execute(query, params)
            policies = []
            
            for row in cursor.fetchall():
                policy = Policy(
                    id=row[0],
                    name=row[1],
                    description=row[2],
                    rules=json.loads(row[3]),
                    priority=row[4],
                    enabled=bool(row[5]),
                    created_at=datetime.fromisoformat(row[6]),
                    updated_at=datetime.fromisoformat(row[7]),
                    expires_at=datetime.fromisoformat(row[8]) if row[8] else None,
                    version=row[9]
                )
                policies.append(policy)
                
            logger.debug(f"Retrieved {len(policies)} policies from cache")
            return policies
            
        except Exception as e:
            logger.error(f"Failed to get all policies: {e}")
            return []
    
    async def delete_policy(self, policy_id: str) -> bool:
        """
        Delete a policy from cache.
        
        Args:
            policy_id: Policy identifier
            
        Returns:
            True if deleted successfully
        """
        try:
            cursor = self._conn.execute(
                "DELETE FROM policies WHERE id = ?", 
                (policy_id,)
            )
            
            if cursor.rowcount > 0:
                logger.info(f"Deleted policy: {policy_id}")
                return True
            else:
                logger.warning(f"Policy not found for deletion: {policy_id}")
                return False
                
        except Exception as e:
            logger.error(f"Failed to delete policy {policy_id}: {e}")
            return False
    
    async def cleanup_expired_policies(self) -> int:
        """
        Remove expired policies from cache.
        
        Returns:
            Number of policies cleaned up
        """
        try:
            cursor = self._conn.execute("""
                DELETE FROM policies 
                WHERE expires_at IS NOT NULL AND expires_at <= ?
            """, (datetime.now().isoformat(),))
            
            count = cursor.rowcount
            if count > 0:
                logger.info(f"Cleaned up {count} expired policies")
                
            return count
            
        except Exception as e:
            logger.error(f"Failed to cleanup expired policies: {e}")
            return 0
    
    async def update_sync_metadata(self, key: str, value: str):
        """Update synchronization metadata"""
        try:
            self._conn.execute("""
                INSERT OR REPLACE INTO sync_metadata (key, value, updated_at)
                VALUES (?, ?, ?)
            """, (key, value, datetime.now().isoformat()))
            
            if key == "last_sync":
                self._last_sync = datetime.fromisoformat(value)
                
        except Exception as e:
            logger.error(f"Failed to update sync metadata {key}: {e}")
    
    async def get_cache_stats(self) -> Dict[str, Any]:
        """Get cache statistics"""
        try:
            # Count total policies
            cursor = self._conn.execute("SELECT COUNT(*) FROM policies")
            total_policies = cursor.fetchone()[0]
            
            # Count enabled policies
            cursor = self._conn.execute("SELECT COUNT(*) FROM policies WHERE enabled = 1")
            enabled_policies = cursor.fetchone()[0]
            
            # Count expired policies
            cursor = self._conn.execute("""
                SELECT COUNT(*) FROM policies 
                WHERE expires_at IS NOT NULL AND expires_at <= ?
            """, (datetime.now().isoformat(),))
            expired_policies = cursor.fetchone()[0]
            
            # Get cache file size
            file_size = self.cache_file.stat().st_size if self.cache_file.exists() else 0
            
            return {
                "total_policies": total_policies,
                "enabled_policies": enabled_policies,
                "expired_policies": expired_policies,
                "last_sync": self._last_sync.isoformat() if self._last_sync else None,
                "cache_file_size_bytes": file_size,
                "cache_file_path": str(self.cache_file)
            }
            
        except Exception as e:
            logger.error(f"Failed to get cache stats: {e}")
            return {}
