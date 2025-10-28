# Quickstart: Resource Limit Scheduling Configuration

**Feature**: 002-resource-limit-scheduling  
**Version**: 1.0  
**Last Updated**: 2025-10-28

---

## Overview

This guide shows how to configure Kubernetes scheduling (nodeSelector and nodeAffinity) for MCP server pods through ResourceLimit configuration.

---

## Prerequisites

- MCP Orchestrator deployed and running
- Kubernetes cluster with labeled nodes
- Access to create ResourceLimits (via gRPC or HTTP API)

---

## Quick Examples

### Example 1: Simple Node Selection by GPU Label

Configure MCP servers to run only on GPU-enabled nodes:

**gRPC Request**:
```json
{
  "name": "gpu-limit",
  "description": "Resource limit for GPU workloads",
  "limits": {
    "cpu": "4000m",
    "memory": "16Gi",
    "nodeSelector": {
      "@type": "type.googleapis.com/mcp.orchestrator.NodeSelector",
      "gpu": "true",
      "gpu-type": "nvidia-v100"
    }
  }
}
```

**HTTP Request**:
```bash
curl -X POST http://orchestrator:8080/namespaces/default/resource-limits \
  -H "Content-Type: application/json" \
  -d '{
    "name": "gpu-limit",
    "description": "GPU nodes only",
    "limits": {
      "cpu": "4000m",
      "memory": "16Gi",
      "nodeSelector": {
        "gpu": "true",
        "gpu-type": "nvidia-v100"
      }
    }
  }'
```

**Result**: MCP server pods will only schedule on nodes with labels `gpu=true` AND `gpu-type=nvidia-v100`.

---

### Example 2: Zone-Specific Deployment with Preference

Deploy in specific zones with preference for one zone:

**gRPC Request**:
```json
{
  "name": "us-west-limit",
  "description": "US West zones with preference for zone 1a",
  "limits": {
    "cpu": "2000m",
    "memory": "4Gi",
    "nodeAffinity": {
      "@type": "type.googleapis.com/mcp.orchestrator.NodeAffinity",
      "nodeAffinity": {
        "requiredDuringSchedulingIgnoredDuringExecution": {
          "nodeSelectorTerms": [
            {
              "matchExpressions": [
                {
                  "key": "topology.kubernetes.io/zone",
                  "operator": "In",
                  "values": ["us-west-1a", "us-west-1b", "us-west-1c"]
                }
              ]
            }
          ]
        },
        "preferredDuringSchedulingIgnoredDuringExecution": [
          {
            "weight": 100,
            "preference": {
              "matchExpressions": [
                {
                  "key": "topology.kubernetes.io/zone",
                  "operator": "In",
                  "values": ["us-west-1a"]
                }
              ]
            }
          }
        ]
      }
    }
  }
}
```

**Result**: 
- MCP server pods MUST be scheduled in us-west-1a, us-west-1b, or us-west-1c
- Scheduler strongly prefers us-west-1a (weight 100)
- If us-west-1a nodes are full, falls back to 1b or 1c

---

### Example 3: Combined NodeSelector and NodeAffinity

Combine simple label matching with complex affinity rules:

```json
{
  "name": "gpu-us-west-limit",
  "description": "GPU nodes in US West region",
  "limits": {
    "cpu": "4000m",
    "memory": "16Gi",
    "nodeSelector": {
      "@type": "type.googleapis.com/mcp.orchestrator.NodeSelector",
      "gpu": "true"
    },
    "nodeAffinity": {
      "@type": "type.googleapis.com/mcp.orchestrator.NodeAffinity",
      "nodeAffinity": {
        "requiredDuringSchedulingIgnoredDuringExecution": {
          "nodeSelectorTerms": [
            {
              "matchExpressions": [
                {
                  "key": "topology.kubernetes.io/region",
                  "operator": "In",
                  "values": ["us-west"]
                }
              ]
            }
          ]
        }
      }
    }
  }
}
```

**Result**: Both conditions must be satisfied:
1. Node has label `gpu=true` (nodeSelector)
2. Node is in `us-west` region (nodeAffinity)

---

## Common Use Cases

### Use Case 1: Dedicated Hardware

Run MCP servers only on nodes with specific hardware:

```json
{
  "nodeSelector": {
    "disktype": "ssd",
    "cpu-type": "intel-xeon"
  }
}
```

### Use Case 2: Environment Isolation

Separate production and development workloads:

```json
{
  "nodeSelector": {
    "environment": "production",
    "tier": "frontend"
  }
}
```

### Use Case 3: High Availability with Zone Spreading

Prefer spreading across multiple zones:

```json
{
  "nodeAffinity": {
    "nodeAffinity": {
      "preferredDuringSchedulingIgnoredDuringExecution": [
        {
          "weight": 50,
          "preference": {
            "matchExpressions": [
              {
                "key": "topology.kubernetes.io/zone",
                "operator": "In",
                "values": ["us-west-1a"]
              }
            ]
          }
        },
        {
          "weight": 50,
          "preference": {
            "matchExpressions": [
              {
                "key": "topology.kubernetes.io/zone",
                "operator": "In",
                "values": ["us-west-1b"]
              }
            ]
          }
        }
      ]
    }
  }
}
```

### Use Case 4: Avoid Specific Nodes

Exclude nodes with certain characteristics:

```json
{
  "nodeAffinity": {
    "nodeAffinity": {
      "requiredDuringSchedulingIgnoredDuringExecution": {
        "nodeSelectorTerms": [
          {
            "matchExpressions": [
              {
                "key": "spot-instance",
                "operator": "DoesNotExist"
              }
            ]
          }
        ]
      }
    }
  }
}
```

---

## Operators Reference

### NodeAffinity Operators

| Operator | Description | Requires Values | Example |
|----------|-------------|-----------------|---------|
| `In` | Label value must be in the list | Yes | `{"key": "zone", "operator": "In", "values": ["a", "b"]}` |
| `NotIn` | Label value must NOT be in the list | Yes | `{"key": "type", "operator": "NotIn", "values": ["spot"]}` |
| `Exists` | Label must exist (any value) | No | `{"key": "gpu", "operator": "Exists"}` |
| `DoesNotExist` | Label must NOT exist | No | `{"key": "spot", "operator": "DoesNotExist"}` |
| `Gt` | Label value must be greater than (numeric) | Yes | `{"key": "cpu-cores", "operator": "Gt", "values": ["8"]}` |
| `Lt` | Label value must be less than (numeric) | Yes | `{"key": "cpu-cores", "operator": "Lt", "values": ["32"]}` |

---

## Validation Rules

### NodeSelector

1. **Keys**: Must follow Kubernetes label format
   - Optional prefix: DNS subdomain (max 253 chars) + `/`
   - Name: 1-63 alphanumeric, dash, underscore, dot
   - Example: `kubernetes.io/hostname`, `region`, `custom.io/type`

2. **Values**: Max 63 chars, alphanumeric + dash/underscore/dot

### NodeAffinity

1. **Required Terms**: At least one term must be satisfied (OR logic)
2. **Match Expressions**: All expressions in a term must be satisfied (AND logic)
3. **Weight**: Must be 1-100 for preferred terms
4. **Operator-Value Relationship**:
   - `In`, `NotIn`, `Gt`, `Lt`: Values required
   - `Exists`, `DoesNotExist`: Values must be empty

---

## Viewing Scheduling Configuration

### Get Resource Limit Details

**gRPC**:
```protobuf
GetResourceLimitRequest {
  name: "gpu-limit"
}
```

**HTTP**:
```bash
curl http://orchestrator:8080/namespaces/default/resource-limits/gpu-limit
```

**Response**:
```json
{
  "name": "gpu-limit",
  "description": "GPU nodes only",
  "limits": {
    "cpu": "4000m",
    "memory": "16Gi",
    "nodeSelector": {
      "gpu": "true",
      "gpu-type": "nvidia-v100"
    }
  },
  "created_at": "2025-10-28T10:00:00Z"
}
```

---

## Updating Scheduling Configuration

Update existing ResourceLimit with new scheduling rules:

```bash
# Note: Implementation may require PUT/PATCH endpoint
curl -X PUT http://orchestrator:8080/namespaces/default/resource-limits/gpu-limit \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated GPU configuration",
    "limits": {
      "cpu": "4000m",
      "memory": "16Gi",
      "nodeSelector": {
        "gpu": "true",
        "gpu-type": "nvidia-a100"
      }
    }
  }'
```

**Important**: Scheduling changes only affect NEW MCP server pods. Existing pods continue with their original configuration.

---

## Troubleshooting

### Pod Stuck in Pending State

**Symptom**: MCP server pod shows `Pending` status

**Check**:
```bash
kubectl describe pod <pod-name> -n <namespace>
```

**Common Causes**:

1. **No Matching Nodes**:
   ```
   Events:
     Warning  FailedScheduling  ... 0/5 nodes are available: 5 node(s) didn't match Pod's node affinity/selector
   ```
   **Solution**: Verify node labels match your selectors
   ```bash
   kubectl get nodes --show-labels
   ```

2. **Invalid Label Format**:
   ```
   Error: Invalid label key format
   ```
   **Solution**: Check label keys follow DNS subdomain format

3. **Conflicting Rules**:
   ```
   Events:
     Warning  FailedScheduling  ... no nodes available
   ```
   **Solution**: Ensure nodeSelector and nodeAffinity rules are compatible

### Verify Node Labels

```bash
# List all node labels
kubectl get nodes --show-labels

# Get specific node labels
kubectl get node <node-name> -o jsonpath='{.metadata.labels}'

# Check if nodes match your selector
kubectl get nodes -l gpu=true,region=us-west
```

### Test Scheduling Configuration

Create a test pod with the same scheduling config:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: test-scheduling
spec:
  nodeSelector:
    gpu: "true"
  affinity:
    nodeAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
        nodeSelectorTerms:
        - matchExpressions:
          - key: topology.kubernetes.io/zone
            operator: In
            values:
            - us-west-1a
            - us-west-1b
  containers:
  - name: test
    image: nginx:latest
```

---

## Best Practices

1. **Start Simple**: Use nodeSelector for straightforward requirements
2. **Use Standard Labels**: Prefer well-known Kubernetes labels
   - `kubernetes.io/hostname`
   - `topology.kubernetes.io/zone`
   - `topology.kubernetes.io/region`
   - `node.kubernetes.io/instance-type`
3. **Test Before Production**: Verify node labels exist before deploying
4. **Document Custom Labels**: Maintain documentation for organization-specific labels
5. **Use Preferred for Optimization**: Soft constraints with preferred terms for performance optimization
6. **Avoid Over-Constraining**: Balance specificity with cluster resource availability

---

## Next Steps

- Review [data-model.md](./data-model.md) for detailed schema
- Check [plan.md](./plan.md) for implementation details
- See Kubernetes documentation for advanced scheduling patterns
- Test with your cluster's node topology

---

## Support

For issues or questions:
- Check pod events: `kubectl describe pod <pod-name>`
- Review orchestrator logs: `kubectl logs -n mcp-orchestrator <orchestrator-pod>`
- Verify node labels: `kubectl get nodes --show-labels`
