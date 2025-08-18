# BEACON Chaincode Integration Guide

## Overview

This document details how the chaincode library components integrate with the Edge Gateway SCS and Administration & Monitoring SCS to provide end-to-end policy enforcement and monitoring.

## Integration Architecture

### 1. Chaincode Library → Edge Gateway SCS Integration

#### Access Control Integration

```python
# gateway-app/src/policy/access_control.py
from chaincode_client import ChaincodeClient

class AccessControlManager:
    def __init__(self, blockchain_client):
        self.chaincode_client = ChaincodeClient(blockchain_client)

    async def check_device_access(self, device_id: str, resource: str, action: str):
        """Check if device has access using access-control chaincode"""
        hashed_device_id = self.hash_device_id(device_id)

        result = await self.chaincode_client.invoke(
            chaincode_id="access_control",
            function="checkAccess",
            args=[self.gateway_id, hashed_device_id, resource, action]
        )

        # Log the decision for monitoring
        await self.log_access_decision(device_id, resource, action, result)
        return result.get('allowed', False)
```

#### Policy Enforcement Integration

```python
# gateway-app/src/policy/rule_engine.py
class PolicyRuleEngine:
    async def evaluate_policy(self, policy_id: str, context: dict):
        """Evaluate policy using policy-enforcement chaincode"""
        result = await self.chaincode_client.invoke(
            chaincode_id="policy_enforcement",
            function="evaluateRule",
            args=[policy_id, json.dumps(context)]
        )
        return result
```

#### Gateway Management Integration

```python
# gateway-app/src/blockchain/gateway_mgmt.py
class GatewayManagement:
    async def register_gateway(self):
        """Register gateway using gateway-management chaincode"""
        result = await self.chaincode_client.invoke(
            chaincode_id="gateway_management",
            function="registerGateway",
            args=[self.gateway_id, self.public_key, self.organization]
        )
        return result

    async def update_status(self, status: str):
        """Update gateway status"""
        await self.chaincode_client.invoke(
            chaincode_id="gateway_management",
            function="updateStatus",
            args=[self.gateway_id, status, str(datetime.utcnow())]
        )
```

#### Device Registry Integration

```python
# gateway-app/src/blockchain/device_registry.py
class DeviceRegistry:
    async def register_device(self, device_id: str, device_type: str, metadata: dict):
        """Register device with privacy preservation"""
        hashed_device_id = self.hash_device_id(device_id)

        # Only store hashed ID and minimal metadata on blockchain
        safe_metadata = {
            "type": device_type,
            "registered_at": str(datetime.utcnow()),
            "gateway_id": self.gateway_id
            # No sensitive device information
        }

        result = await self.chaincode_client.invoke(
            chaincode_id="device_registry",
            function="registerDevice",
            args=[hashed_device_id, json.dumps(safe_metadata)]
        )
        return result
```

#### Audit Logging Integration

```python
# gateway-app/src/blockchain/audit_logger.py
class AuditLogger:
    async def log_policy_enforcement(self, device_id: str, policy_id: str,
                                   action: str, result: str, details: dict):
        """Log policy enforcement with privacy preservation"""
        hashed_device_id = self.hash_device_id(device_id)

        # Create privacy-preserving audit entry
        audit_entry = {
            "event_type": "policy_enforcement",
            "gateway_id": self.gateway_id,
            "hashed_device_id": hashed_device_id,
            "policy_id": policy_id,
            "action": action,
            "result": result,
            "timestamp": str(datetime.utcnow()),
            "details_hash": hashlib.sha256(json.dumps(details).encode()).hexdigest()
        }

        await self.chaincode_client.invoke(
            chaincode_id="audit_logging",
            function="logEvent",
            args=[json.dumps(audit_entry)]
        )
```

### 2. Frontend Integration with Chaincode Data

#### I&O SCS Operator Components

##### PolicyEditor.jsx

```javascript
// admin-monitoring-scs/web-client/src/components/io-scs/PolicyEditor.jsx
import React, { useState } from "react";
import { useChaincode } from "../../hooks/useChaincode";

const PolicyEditor = () => {
  const { invokeChaincode } = useChaincode();
  const [policy, setPolicy] = useState({
    name: "",
    rules: [],
    targetGateways: [],
    effect: "allow",
  });

  const handleSavePolicy = async () => {
    try {
      await invokeChaincode("access_control", "createPolicy", [
        policy.name,
        JSON.stringify(policy.rules),
        JSON.stringify(policy.targetGateways),
        policy.effect,
      ]);
      // Show success notification
    } catch (error) {
      // Handle error
    }
  };

  return (
    <div className="policy-editor">
      <h2>Create/Edit Policy</h2>
      <form onSubmit={handleSavePolicy}>{/* Policy creation form */}</form>
    </div>
  );
};
```

##### AccessControlMonitor.jsx

```javascript
// admin-monitoring-scs/web-client/src/components/io-scs/AccessControlMonitor.jsx
import React, { useEffect, useState } from "react";
import { useChaincode } from "../../hooks/useChaincode";
import { MetricsCard } from "../common/Charts/MetricsCard";

const AccessControlMonitor = () => {
  const { queryChaincode } = useChaincode();
  const [metrics, setMetrics] = useState({
    totalPolicies: 0,
    activeGateways: 0,
    enforcementRate: 0,
    violationsToday: 0,
  });

  useEffect(() => {
    const loadMetrics = async () => {
      const [policies, gateways, enforcement] = await Promise.all([
        queryChaincode("access_control", "getPolicyCount"),
        queryChaincode("gateway_management", "getActiveGatewayCount"),
        queryChaincode("audit_logging", "getEnforcementStats", ["24h"]),
      ]);

      setMetrics({
        totalPolicies: policies.count,
        activeGateways: gateways.count,
        enforcementRate: enforcement.successRate,
        violationsToday: enforcement.violations,
      });
    };

    loadMetrics();
    const interval = setInterval(loadMetrics, 30000); // Update every 30s
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="access-control-monitor">
      <h2>Access Control Overview</h2>
      <div className="metrics-grid">
        <MetricsCard
          title="Total Policies"
          value={metrics.totalPolicies}
          trend="+5 this week"
        />
        <MetricsCard
          title="Active Gateways"
          value={metrics.activeGateways}
          status="healthy"
        />
        <MetricsCard
          title="Enforcement Rate"
          value={`${metrics.enforcementRate}%`}
          trend="↑ 2.3%"
        />
        <MetricsCard
          title="Violations Today"
          value={metrics.violationsToday}
          status={metrics.violationsToday > 10 ? "warning" : "normal"}
        />
      </div>
    </div>
  );
};
```

#### Gateway Operator Components

##### PolicyEnforcementLogs.jsx

```javascript
// admin-monitoring-scs/web-client/src/components/gateway/PolicyEnforcementLogs.jsx
import React, { useEffect, useState } from "react";
import { DataTable } from "../common/DataTable";
import { useGatewayApi } from "../../hooks/useGatewayApi";

const PolicyEnforcementLogs = () => {
  const { getEnforcementLogs } = useGatewayApi();
  const [logs, setLogs] = useState([]);

  useEffect(() => {
    const loadLogs = async () => {
      const enforcementLogs = await getEnforcementLogs();
      setLogs(enforcementLogs);
    };

    loadLogs();
    const interval = setInterval(loadLogs, 5000); // Real-time updates
    return () => clearInterval(interval);
  }, []);

  const columns = [
    { key: "timestamp", title: "Time" },
    { key: "deviceId", title: "Device ID" }, // Shows real device ID for gateway operator
    { key: "action", title: "Action" },
    { key: "resource", title: "Resource" },
    {
      key: "result",
      title: "Result",
      render: (value) => (
        <span className={`status ${value}`}>
          {value === "allowed" ? "✅ Allow" : "❌ Deny"}
        </span>
      ),
    },
    { key: "policyId", title: "Policy Applied" },
  ];

  return (
    <div className="policy-enforcement-logs">
      <h2>Policy Enforcement Log</h2>
      <DataTable
        data={logs}
        columns={columns}
        pagination={true}
        realTime={true}
      />
    </div>
  );
};
```

##### DeviceRegistration.jsx

```javascript
// admin-monitoring-scs/web-client/src/components/gateway/DeviceRegistration.jsx
import React, { useState } from "react";
import { useGatewayApi } from "../../hooks/useGatewayApi";
import { InputField, SelectField } from "../common/Forms";

const DeviceRegistration = () => {
  const { registerDevice } = useGatewayApi();
  const [device, setDevice] = useState({
    id: "",
    type: "",
    name: "",
    location: "",
  });

  const handleRegister = async (e) => {
    e.preventDefault();
    try {
      // This will call device-registry chaincode with privacy hashing
      await registerDevice(device);
      // Show success message
    } catch (error) {
      // Handle error
    }
  };

  return (
    <div className="device-registration">
      <h2>Register New Device</h2>
      <form onSubmit={handleRegister}>
        <InputField
          label="Device ID"
          value={device.id}
          onChange={(value) => setDevice({ ...device, id: value })}
          required
        />
        <SelectField
          label="Device Type"
          value={device.type}
          options={[
            { value: "sensor", label: "Sensor" },
            { value: "actuator", label: "Actuator" },
            { value: "gateway", label: "Gateway" },
          ]}
          onChange={(value) => setDevice({ ...device, type: value })}
          required
        />
        <button type="submit">Register Device</button>
      </form>

      <div className="privacy-notice">
        <p>
          🔒 Your device information is kept private. Only hashed identifiers
          are shared with the blockchain.
        </p>
      </div>
    </div>
  );
};
```

### 3. Grafana Dashboard Integration

#### policy-enforcement.json (Gateway Dashboard)

```json
{
  "dashboard": {
    "title": "Policy Enforcement",
    "panels": [
      {
        "title": "Real-time Access Decisions",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(gateway_access_decisions_total[5m])",
            "legendFormat": "{{result}}"
          }
        ]
      },
      {
        "title": "Policy Enforcement Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, gateway_policy_enforcement_duration_seconds)",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "Device Registration Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(gateway_device_registrations_total[1h])",
            "legendFormat": "Registrations/hour"
          }
        ]
      }
    ]
  }
}
```

#### access-control-analytics.json (I&O SCS Dashboard)

```json
{
  "dashboard": {
    "title": "Access Control Analytics",
    "panels": [
      {
        "title": "Chaincode Execution Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(chaincode_execution_duration_seconds[5m])",
            "legendFormat": "{{chaincode_id}}"
          }
        ]
      },
      {
        "title": "Policy Violations by Gateway",
        "type": "heatmap",
        "targets": [
          {
            "expr": "increase(policy_violations_total[1h])",
            "legendFormat": "{{gateway_hash}}"
          }
        ]
      }
    ]
  }
}
```

### 4. Privacy-Preserving Data Flow

#### Data Masking Example

```javascript
// admin-monitoring-scs/web-client/src/components/common/Privacy/DataMask.jsx
import React from "react";

const DataMask = ({
  data,
  hashingEnabled,
  showToOperator,
  showHashToIOSCS,
}) => {
  if (showToOperator) {
    // Gateway operators see real data
    return <span className="data-clear">{data}</span>;
  } else if (showHashToIOSCS && hashingEnabled) {
    // I&O SCS operators see hashed data
    const hashedData = `sha256:${data.substring(0, 8)}...`;
    return (
      <span className="data-hashed" title="Hashed for privacy">
        {hashedData}
      </span>
    );
  } else {
    // No access
    return <span className="data-hidden">***HIDDEN***</span>;
  }
};

export default DataMask;
```

### 5. Real-time Updates

#### WebSocket Integration for Policy Updates

```javascript
// admin-monitoring-scs/web-client/src/hooks/useRealTime.js
import { useEffect, useState } from "react";
import { useWebSocket } from "./useWebSocket";

export const useRealTimePolicyUpdates = () => {
  const [policyUpdates, setPolicyUpdates] = useState([]);
  const { socket } = useWebSocket("/policy-updates");

  useEffect(() => {
    if (socket) {
      socket.on("policy_created", (policy) => {
        setPolicyUpdates((prev) => [
          ...prev,
          {
            type: "created",
            policy,
            timestamp: new Date(),
          },
        ]);
      });

      socket.on("policy_enforced", (enforcement) => {
        setPolicyUpdates((prev) => [
          ...prev,
          {
            type: "enforced",
            enforcement,
            timestamp: new Date(),
          },
        ]);
      });

      socket.on("policy_violation", (violation) => {
        setPolicyUpdates((prev) => [
          ...prev,
          {
            type: "violation",
            violation,
            timestamp: new Date(),
          },
        ]);
      });
    }

    return () => {
      if (socket) {
        socket.off("policy_created");
        socket.off("policy_enforced");
        socket.off("policy_violation");
      }
    };
  }, [socket]);

  return policyUpdates;
};
```

## Implementation Priority

1. **Phase 1**: Core chaincode integration in Edge Gateway
2. **Phase 2**: Basic policy management UI components
3. **Phase 3**: Real-time monitoring and enforcement dashboards
4. **Phase 4**: Advanced privacy features and compliance reporting

This integration ensures that policy enforcement is transparent to end users while maintaining privacy and providing comprehensive monitoring capabilities.
