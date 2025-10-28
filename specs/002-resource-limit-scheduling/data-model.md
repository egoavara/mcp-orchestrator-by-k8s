# Data Model: Resource Limit Scheduling Configuration

**Feature**: 002-resource-limit-scheduling  
**Date**: 2025-10-28  
**Status**: Design Phase

---

## Overview

This document defines the data structures for storing and managing Kubernetes scheduling configuration (nodeSelector and nodeAffinity) as part of ResourceLimit entities.

---

## Entity: ResourceLimit (Enhanced)

### Purpose

Represents resource constraints and scheduling configuration for MCP server pods. Controls CPU, memory, storage limits AND pod placement on Kubernetes nodes.

### Attributes

| Attribute | Type | Required | Description | Validation |
|-----------|------|----------|-------------|------------|
| name | string | Yes | Unique identifier | 1-253 chars, DNS subdomain format |
| description | string | No | Human-readable description | Max 2048 chars |
| labels | map<string, string> | No | User-defined labels | Valid Kubernetes label format |
| cpu | string | Yes | CPU request | Kubernetes quantity format (e.g., "1000m", "2") |
| memory | string | Yes | Memory request | Kubernetes quantity format (e.g., "512Mi", "1Gi") |
| cpu_limit | string | No | CPU limit | Kubernetes quantity format, defaults to cpu |
| memory_limit | string | No | Memory limit | Kubernetes quantity format, defaults to memory |
| ephemeral_storage | string | No | Ephemeral storage request | Kubernetes quantity format |
| volumes | map<string, VolumeLimit> | No | Persistent volume configuration | Volume name → VolumeLimit |
| **node_selector** | **map<string, string>** | **No** | **Simple node label selectors** | **Valid label key-value pairs** |
| **node_affinity** | **NodeAffinity** | **No** | **Complex scheduling rules** | **Valid affinity structure** |
| created_at | timestamp | Auto | Creation timestamp | ISO 8601 |
| deleted_at | timestamp | Auto | Deletion timestamp (soft delete) | ISO 8601 |

### State Transitions

```text
[Created] → [Active] → [Marked for Deletion] → [Deleted]
             ↓
        [Updated] (scheduling config can be modified)
```

- **Created**: Initial state after successful creation
- **Active**: Available for use by MCP templates
- **Updated**: Scheduling configuration modified (applies to new pods only)
- **Marked for Deletion**: Finalizer prevents deletion if MCP templates depend on it
- **Deleted**: Removed from storage

### Relationships

- **One-to-Many** with McpTemplate: A ResourceLimit can be used by multiple MCP templates
- **Belongs-To** Namespace: ResourceLimit exists within a Kubernetes namespace
- **Dependency Check**: Cannot be deleted if any MCP template references it

---

## Entity: SchedulingConfig (New)

### Purpose

Encapsulates Kubernetes scheduling configuration extracted from ResourceLimit.

### Structure (Rust)

```rust
pub struct SchedulingConfig {
    pub node_selector: Option<BTreeMap<String, String>>,
    pub node_affinity: Option<Affinity>,
}
```

### Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| node_selector | BTreeMap<string, string> | No | Simple key-value label selectors |
| node_affinity | Affinity | No | Complex scheduling rules with match expressions |

### Serialization

**Storage Format** (ConfigMap data fields):
```yaml
node_selector: '{"region":"us-west","gpu":"true"}'
node_affinity: '{"nodeAffinity":{"requiredDuringSchedulingIgnoredDuringExecution":{...}}}'
```

Both stored as JSON strings for debuggability and compatibility with kubectl.

---

## Entity: NodeAffinity

### Purpose

Defines complex node scheduling rules with required and preferred constraints.

### Structure (k8s-openapi types)

```rust
pub struct Affinity {
    pub node_affinity: Option<NodeAffinity>,
    pub pod_affinity: Option<PodAffinity>,          // Unused
    pub pod_anti_affinity: Option<PodAntiAffinity>, // Unused
}

pub struct NodeAffinity {
    pub required_during_scheduling_ignored_during_execution: Option<NodeSelector>,
    pub preferred_during_scheduling_ignored_during_execution: Option<Vec<PreferredSchedulingTerm>>,
}

pub struct NodeSelector {
    pub node_selector_terms: Vec<NodeSelectorTerm>, // Terms are ORed
}

pub struct NodeSelectorTerm {
    pub match_expressions: Option<Vec<NodeSelectorRequirement>>, // Expressions are ANDed
    pub match_fields: Option<Vec<NodeSelectorRequirement>>,
}

pub struct NodeSelectorRequirement {
    pub key: String,
    pub operator: String, // "In", "NotIn", "Exists", "DoesNotExist", "Gt", "Lt"
    pub values: Option<Vec<String>>,
}

pub struct PreferredSchedulingTerm {
    pub preference: NodeSelectorTerm,
    pub weight: i32, // 1-100
}
```

### Validation Rules

1. **Required Terms**:
   - At least one NodeSelectorTerm must be provided if required section exists
   - All match expressions in a term must be satisfied (AND logic)
   - At least one term must be satisfied (OR logic)

2. **Preferred Terms**:
   - Weight must be 1-100
   - Scheduler sums weights of all satisfied preferences
   - Does not prevent scheduling if not satisfied

3. **Match Expressions**:
   - Operator must be: In, NotIn, Exists, DoesNotExist, Gt, Lt
   - Values required for: In, NotIn, Gt, Lt
   - Values forbidden for: Exists, DoesNotExist

---

## Storage Schema

### ConfigMap Structure

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: resource-limit-rl-example
  namespace: mcp-orchestrator
  labels:
    mcp-orchestrator.egoavara.net/type: resource-limit
    mcp-orchestrator.egoavara.net/managed: "true"
  annotations:
    mcp-orchestrator.egoavara.net/description: "GPU nodes in US West"
data:
  # Existing fields
  cpu: "2000m"
  memory: "4Gi"
  cpu_limit: "4000m"
  memory_limit: "8Gi"
  ephemeral_storage: "10Gi"
  volumes: '{"data":{"size":"10Gi","storage_class":"fast"}}'
  
  # New scheduling fields
  node_selector: '{"gpu":"true","region":"us-west"}'
  node_affinity: |
    {
      "nodeAffinity": {
        "requiredDuringSchedulingIgnoredDuringExecution": {
          "nodeSelectorTerms": [
            {
              "matchExpressions": [
                {
                  "key": "topology.kubernetes.io/zone",
                  "operator": "In",
                  "values": ["us-west-1a", "us-west-1b"]
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
                  "key": "node.kubernetes.io/instance-type",
                  "operator": "In",
                  "values": ["g4dn.xlarge", "g4dn.2xlarge"]
                }
              ]
            }
          }
        ]
      }
    }
```

### Storage Constants (Rust)

```rust
// In store_resource_limit.rs
const DATA_NODE_SELECTOR: &str = "node_selector";
const DATA_NODE_AFFINITY: &str = "node_affinity";
```

---

## Protobuf Schema

### resource_limit.proto

```protobuf
syntax = "proto3";

package mcp.orchestrator.v1;

import "google/protobuf/any.proto";
import "common.proto";

message CreateResourceLimitRequest {
  string name = 1;
  string description = 3;
  ResourceLimit limits = 6;
  map<string, string> labels = 5;
}

message GetResourceLimitRequest {
  string name = 1;
}

message ListResourceLimitsRequest {
  LabelQuery label = 1;
  optional int32 first = 2;
  optional string after = 3;
}

message ListResourceLimitsResponse {
  repeated ResourceLimitResponse data = 1;
  optional string endCursor = 2;
  bool hasNextPage = 3;
}

message DeleteResourceLimitRequest {
  string name = 1;
  bool force = 2;
}

message DeleteResourceLimitResponse {
  bool success = 1;
  string message = 2;
}

message ResourceLimitResponse {
  string name = 1;
  string description = 3;
  ResourceLimit limits = 4;
  map<string, string> labels = 5;
  string created_at = 6;
  optional string deleted_at = 7;
}

// Enhanced ResourceLimit message
message ResourceLimit {
  string cpu = 1;
  string memory = 2;
  optional string cpu_limit = 3;
  optional string memory_limit = 4;
  optional string ephemeral_storage = 5;
  map<string, VolumeLimit> volumes = 6;
  
  // NEW: Scheduling configuration
  optional google.protobuf.Any node_selector = 7;
  optional google.protobuf.Any node_affinity = 8;
}
```

### Type URL Convention

When encoding scheduling configuration in protobuf Any:

- **NodeSelector**: `type_url: "type.googleapis.com/mcp.orchestrator.NodeSelector"`
- **NodeAffinity**: `type_url: "type.googleapis.com/mcp.orchestrator.NodeAffinity"`

However, in practice, these will be extracted as JSON and stored directly in ConfigMap, bypassing complex Any encoding.

---

## Frontend Data Model

### TypeScript/Rust Model (Yew Frontend)

```rust
// In crates/mcp-orchestrator-front/src/models/resource_limit.rs

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimit {
    pub name: String,
    pub description: Option<String>,
    pub labels: HashMap<String, String>,
    pub cpu: String,
    pub memory: String,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub ephemeral_storage: Option<String>,
    pub volumes: HashMap<String, VolumeLimit>,
    
    // NEW
    #[serde(default)]
    pub node_selector: Option<HashMap<String, String>>,
    
    #[serde(default)]
    pub node_affinity: Option<NodeAffinityConfig>,
    
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeAffinityConfig {
    pub required_terms: Option<Vec<NodeSelectorTerm>>,
    pub preferred_terms: Option<Vec<PreferredSchedulingTerm>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeSelectorTerm {
    pub match_expressions: Vec<NodeSelectorRequirement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeSelectorRequirement {
    pub key: String,
    pub operator: String,
    pub values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreferredSchedulingTerm {
    pub weight: i32,
    pub preference: NodeSelectorTerm,
}
```

---

## Validation Schema

### NodeSelector Validation

```rust
fn validate_node_selector(selector: &BTreeMap<String, String>) -> Result<(), ValidationError> {
    if selector.is_empty() {
        return Err(ValidationError::EmptyNodeSelector);
    }
    
    for (key, value) in selector {
        validate_label_key(key)?;
        validate_label_value(value)?;
    }
    
    Ok(())
}

fn validate_label_key(key: &str) -> Result<(), ValidationError> {
    // Format: [prefix/]name
    // Prefix: DNS subdomain (optional), max 253 chars
    // Name: 1-63 chars, alphanumeric + dash/underscore/dot, start/end alphanumeric
    
    let parts: Vec<&str> = key.split('/').collect();
    match parts.len() {
        1 => validate_label_name(parts[0]),
        2 => {
            validate_dns_subdomain(parts[0])?;
            validate_label_name(parts[1])
        },
        _ => Err(ValidationError::InvalidLabelKey(key.to_string())),
    }
}

fn validate_label_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > 63 {
        return Err(ValidationError::LabelNameLength(name.len()));
    }
    
    let regex = Regex::new(r"^[a-zA-Z0-9]([-_.a-zA-Z0-9]*[a-zA-Z0-9])?$").unwrap();
    if !regex.is_match(name) {
        return Err(ValidationError::LabelNameFormat(name.to_string()));
    }
    
    Ok(())
}

fn validate_label_value(value: &str) -> Result<(), ValidationError> {
    if value.len() > 63 {
        return Err(ValidationError::LabelValueLength(value.len()));
    }
    
    if value.is_empty() {
        return Ok(()); // Empty value allowed
    }
    
    let regex = Regex::new(r"^[a-zA-Z0-9]([-_.a-zA-Z0-9]*[a-zA-Z0-9])?$").unwrap();
    if !regex.is_match(value) {
        return Err(ValidationError::LabelValueFormat(value.to_string()));
    }
    
    Ok(())
}
```

### NodeAffinity Validation

```rust
fn validate_node_affinity(affinity: &Affinity) -> Result<(), ValidationError> {
    let Some(node_affinity) = &affinity.node_affinity else {
        return Ok(());
    };
    
    // Validate required terms
    if let Some(required) = &node_affinity.required_during_scheduling_ignored_during_execution {
        if required.node_selector_terms.is_empty() {
            return Err(ValidationError::EmptyRequiredTerms);
        }
        for term in &required.node_selector_terms {
            validate_node_selector_term(term)?;
        }
    }
    
    // Validate preferred terms
    if let Some(preferred) = &node_affinity.preferred_during_scheduling_ignored_during_execution {
        for pref_term in preferred {
            if pref_term.weight < 1 || pref_term.weight > 100 {
                return Err(ValidationError::InvalidWeight(pref_term.weight));
            }
            validate_node_selector_term(&pref_term.preference)?;
        }
    }
    
    Ok(())
}

fn validate_node_selector_term(term: &NodeSelectorTerm) -> Result<(), ValidationError> {
    let has_expressions = term.match_expressions.as_ref().map_or(false, |e| !e.is_empty());
    let has_fields = term.match_fields.as_ref().map_or(false, |f| !f.is_empty());
    
    if !has_expressions && !has_fields {
        return Err(ValidationError::EmptyNodeSelectorTerm);
    }
    
    if let Some(expressions) = &term.match_expressions {
        for expr in expressions {
            validate_node_selector_requirement(expr)?;
        }
    }
    
    if let Some(fields) = &term.match_fields {
        for field in fields {
            validate_node_selector_requirement(field)?;
        }
    }
    
    Ok(())
}

fn validate_node_selector_requirement(req: &NodeSelectorRequirement) -> Result<(), ValidationError> {
    validate_label_key(&req.key)?;
    
    let valid_operators = ["In", "NotIn", "Exists", "DoesNotExist", "Gt", "Lt"];
    if !valid_operators.contains(&req.operator.as_str()) {
        return Err(ValidationError::InvalidOperator(req.operator.clone()));
    }
    
    // Validate values based on operator
    match req.operator.as_str() {
        "In" | "NotIn" | "Gt" | "Lt" => {
            if req.values.as_ref().map_or(true, |v| v.is_empty()) {
                return Err(ValidationError::MissingValues(req.operator.clone()));
            }
            // Validate each value
            if let Some(values) = &req.values {
                for value in values {
                    validate_label_value(value)?;
                }
            }
        },
        "Exists" | "DoesNotExist" => {
            if req.values.as_ref().map_or(false, |v| !v.is_empty()) {
                return Err(ValidationError::UnexpectedValues(req.operator.clone()));
            }
        },
        _ => {}
    }
    
    Ok(())
}
```

---

## Example Use Cases

### Use Case 1: GPU Nodes with Simple NodeSelector

```json
{
  "name": "gpu-limit",
  "description": "Resource limit for GPU workloads",
  "limits": {
    "cpu": "4000m",
    "memory": "16Gi",
    "nodeSelector": {
      "gpu": "true",
      "gpu-type": "nvidia-v100"
    }
  }
}
```

**Result**: MCP server pods will only schedule on nodes with both labels `gpu=true` AND `gpu-type=nvidia-v100`.

### Use Case 2: Zone-Specific with NodeAffinity

```json
{
  "name": "us-west-limit",
  "description": "US West zone with preference for 1a",
  "limits": {
    "cpu": "2000m",
    "memory": "4Gi",
    "nodeAffinity": {
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

**Result**: MCP server pods MUST be in us-west zones, with strong preference for us-west-1a.

### Use Case 3: Combined NodeSelector and NodeAffinity

```json
{
  "name": "gpu-us-west-limit",
  "limits": {
    "cpu": "4000m",
    "memory": "16Gi",
    "nodeSelector": {
      "gpu": "true"
    },
    "nodeAffinity": {
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

**Result**: MCP server pods must satisfy BOTH:
1. Node has label `gpu=true` (nodeSelector)
2. Node is in region `us-west` (nodeAffinity required)

---

## Summary

This data model extends ResourceLimit with optional scheduling configuration while maintaining backward compatibility. The design follows existing patterns (JSON storage in ConfigMap), leverages k8s-openapi types for correctness, and provides comprehensive validation at creation time.
