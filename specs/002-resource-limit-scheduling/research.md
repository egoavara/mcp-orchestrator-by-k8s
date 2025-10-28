# Research: Resource Limit Scheduling Configuration

**Feature**: 002-resource-limit-scheduling  
**Date**: 2025-10-28  
**Purpose**: Technical research to resolve unknowns before design phase

---

## 1. Protobuf Any Type Handling in Rust

### Decision

Use `prost-wkt-types` crate which is already in dependencies to handle `google.protobuf.Any`. Store scheduling configuration as JSON strings in ConfigMap data fields, bypassing complex Any serialization.

### Rationale

1. **Simplification**: Instead of fully encoding/decoding protobuf Any with type URLs, we:
   - Accept Any in gRPC API (client flexibility)
   - Extract JSON value from Any
   - Store JSON string directly in ConfigMap
   - Deserialize to k8s-openapi types only when creating pods

2. **Existing Pattern**: Current codebase already uses JSON serialization for complex fields:
   ```rust
   // From store_resource_limit.rs line 59
   volumes: parse_data_elem(&cm.data, DATA_VOLUMES)?,
   ```
   This pattern deserializes JSON strings from ConfigMap data using serde.

3. **Type Safety**: Deserialization to k8s-openapi types happens in Rust code, providing compile-time type safety where it matters (pod creation).

### Implementation Strategy

```rust
// In ResourceLimitData
pub struct SchedulingConfig {
    pub node_selector: Option<BTreeMap<String, String>>,
    pub node_affinity: Option<k8s_openapi::api::core::v1::Affinity>,
}

// Serialization for storage
impl SchedulingConfig {
    fn to_json_strings(&self) -> Result<(Option<String>, Option<String>), AppError> {
        let node_selector_json = self.node_selector.as_ref()
            .map(|ns| serde_json::to_string(ns))
            .transpose()?;
        
        let node_affinity_json = self.node_affinity.as_ref()
            .map(|na| serde_json::to_string(na))
            .transpose()?;
        
        Ok((node_selector_json, node_affinity_json))
    }
    
    fn from_json_strings(
        node_selector: Option<String>,
        node_affinity: Option<String>
    ) -> Result<Self, AppError> {
        Ok(Self {
            node_selector: node_selector
                .map(|s| serde_json::from_str(&s))
                .transpose()?,
            node_affinity: node_affinity
                .map(|s| serde_json::from_str(&s))
                .transpose()?,
        })
    }
}
```

### Alternatives Considered

1. **Full Any Protocol**: Implement proper protobuf Any encoding with type URLs
   - **Rejected**: Overly complex, requires type registry, not needed for internal storage
   
2. **Define Full Kubernetes Types in Protobuf**: Create complete protobuf schema for NodeAffinity
   - **Rejected**: Violates user requirement ("protobuf 정의에 nodeselector 를 전부 정의하지는 말고")
   - Would duplicate Kubernetes API definitions
   
3. **Custom Binary Encoding**: Use bincode or similar for Rust struct serialization
   - **Rejected**: JSON is more debuggable, compatible with kubectl, human-readable

---

## 2. Kubernetes Scheduling Configuration

### NodeSelector Structure

**Type**: `BTreeMap<String, String>` (ordered map for deterministic serialization)

**Format**:
```json
{
  "region": "us-west",
  "gpu": "true",
  "topology.kubernetes.io/zone": "us-west-1a"
}
```

**Validation Rules** (per Kubernetes label requirements):

1. **Keys**:
   - Optional prefix: DNS subdomain (max 253 chars) + `/`
   - Name: alphanumeric, dash, underscore, dot (max 63 chars)
   - Must start/end with alphanumeric
   - Example: `kubernetes.io/hostname`, `region`, `custom.io/gpu-type`

2. **Values**:
   - Max 63 characters
   - Alphanumeric, dash, underscore, dot
   - Must start/end with alphanumeric
   - Empty string allowed

**Rust Type**: `BTreeMap<String, String>` (k8s-openapi doesn't have dedicated NodeSelector type)

### NodeAffinity Structure

**Type**: `k8s_openapi::api::core::v1::Affinity` → `node_affinity` field

**Complete Structure**:
```rust
pub struct Affinity {
    pub node_affinity: Option<NodeAffinity>,
    pub pod_affinity: Option<PodAffinity>,     // Not used in our case
    pub pod_anti_affinity: Option<PodAntiAffinity>, // Not used
}

pub struct NodeAffinity {
    pub required_during_scheduling_ignored_during_execution: Option<NodeSelector>,
    pub preferred_during_scheduling_ignored_during_execution: Option<Vec<PreferredSchedulingTerm>>,
}

pub struct NodeSelector {
    pub node_selector_terms: Vec<NodeSelectorTerm>,
}

pub struct NodeSelectorTerm {
    pub match_expressions: Option<Vec<NodeSelectorRequirement>>,
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

**JSON Example**:
```json
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
              "key": "gpu",
              "operator": "Exists"
            }
          ]
        }
      }
    ]
  }
}
```

**Validation Rules**:

1. **Operators**: Must be one of: `In`, `NotIn`, `Exists`, `DoesNotExist`, `Gt`, `Lt`
2. **Values**: Required for `In`, `NotIn`, `Gt`, `Lt`; must be absent for `Exists`, `DoesNotExist`
3. **Weight**: 1-100 for preferred terms
4. **Terms Logic**: Multiple terms are ORed, multiple expressions within a term are ANDed

### Decision

Store full Affinity structure but only populate `node_affinity` field. This allows future extension to pod affinity/anti-affinity if needed, while maintaining compatibility with k8s-openapi types.

---

## 3. Validation Strategy

### Decision

**Two-tier validation**:
1. **Basic validation** in Rust before storage (label format, required fields)
2. **Full validation** delegated to Kubernetes API during pod creation

### Rationale

1. **Early Feedback**: Catch obvious errors (invalid label keys, missing required values) immediately
2. **Correctness**: Kubernetes API is the source of truth for scheduling validity
3. **Future-Proof**: New K8s scheduling features work without code changes
4. **Error Context**: Can provide better error messages during resource limit creation vs pod creation

### Basic Validation Implementation

```rust
// Validate label key format
fn validate_label_key(key: &str) -> Result<(), AppError> {
    let parts: Vec<&str> = key.split('/').collect();
    
    match parts.len() {
        1 => validate_label_name(parts[0]),
        2 => {
            validate_dns_subdomain(parts[0])?;
            validate_label_name(parts[1])
        },
        _ => Err(AppError::InvalidInput(
            "Label key must be [prefix/]name format".into()
        ))
    }
}

fn validate_label_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() || name.len() > 63 {
        return Err(AppError::InvalidInput("Label name must be 1-63 chars".into()));
    }
    
    let re = regex::Regex::new(r"^[a-zA-Z0-9]([-_.\w]*[a-zA-Z0-9])?$").unwrap();
    if !re.is_match(name) {
        return Err(AppError::InvalidInput(
            "Label name must be alphanumeric, dash, underscore, dot".into()
        ));
    }
    
    Ok(())
}

fn validate_node_selector(selector: &BTreeMap<String, String>) -> Result<(), AppError> {
    for (key, value) in selector {
        validate_label_key(key)?;
        validate_label_value(value)?;
    }
    Ok(())
}

fn validate_node_affinity(affinity: &Affinity) -> Result<(), AppError> {
    if let Some(node_affinity) = &affinity.node_affinity {
        // Validate required terms
        if let Some(required) = &node_affinity.required_during_scheduling_ignored_during_execution {
            for term in &required.node_selector_terms {
                validate_node_selector_term(term)?;
            }
        }
        
        // Validate preferred terms
        if let Some(preferred) = &node_affinity.preferred_during_scheduling_ignored_during_execution {
            for pref_term in preferred {
                if pref_term.weight < 1 || pref_term.weight > 100 {
                    return Err(AppError::InvalidInput(
                        "Preferred term weight must be 1-100".into()
                    ));
                }
                validate_node_selector_term(&pref_term.preference)?;
            }
        }
    }
    Ok(())
}

fn validate_node_selector_term(term: &NodeSelectorTerm) -> Result<(), AppError> {
    if let Some(expressions) = &term.match_expressions {
        for expr in expressions {
            validate_node_selector_requirement(expr)?;
        }
    }
    Ok(())
}

fn validate_node_selector_requirement(req: &NodeSelectorRequirement) -> Result<(), AppError> {
    validate_label_key(&req.key)?;
    
    let valid_operators = ["In", "NotIn", "Exists", "DoesNotExist", "Gt", "Lt"];
    if !valid_operators.contains(&req.operator.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Invalid operator: {}. Must be one of: {:?}",
            req.operator, valid_operators
        )));
    }
    
    // Check values presence based on operator
    match req.operator.as_str() {
        "In" | "NotIn" | "Gt" | "Lt" => {
            if req.values.as_ref().map_or(true, |v| v.is_empty()) {
                return Err(AppError::InvalidInput(format!(
                    "Operator {} requires values", req.operator
                )));
            }
        },
        "Exists" | "DoesNotExist" => {
            if req.values.as_ref().map_or(false, |v| !v.is_empty()) {
                return Err(AppError::InvalidInput(format!(
                    "Operator {} must not have values", req.operator
                )));
            }
        },
        _ => {}
    }
    
    Ok(())
}
```

### Alternatives Considered

1. **No Validation**: Let Kubernetes reject invalid configs
   - **Rejected**: Poor user experience, errors appear during pod creation not config creation
   
2. **Full Validation**: Replicate all Kubernetes validation logic
   - **Rejected**: Complex, maintenance burden, version skew issues

---

## 4. Backward Compatibility Strategy

### Decision

Make scheduling fields completely optional at all layers. Default behavior is "no scheduling constraints" (equivalent to current behavior).

### Implementation

1. **Protobuf**: Use `optional` for new fields
   ```protobuf
   optional google.protobuf.Any node_selector = 7;
   optional google.protobuf.Any node_affinity = 8;
   ```

2. **Storage**: Use `Option<String>` for JSON fields in ConfigMap
   ```rust
   pub struct ResourceLimitData {
       // existing fields...
       pub scheduling: Option<SchedulingConfig>,
   }
   ```

3. **Deserialization**: Handle missing fields gracefully
   ```rust
   let scheduling = match (
       parse_data_elem::<String>(&cm.data, DATA_NODE_SELECTOR).ok().flatten(),
       parse_data_elem::<String>(&cm.data, DATA_NODE_AFFINITY).ok().flatten(),
   ) {
       (None, None) => None,
       (ns, na) => Some(SchedulingConfig::from_json_strings(ns, na)?),
   };
   ```

4. **Pod Creation**: Only apply scheduling if present
   ```rust
   if let Some(scheduling) = &resource_limit.scheduling {
       if let Some(ref node_selector) = scheduling.node_selector {
           pod_spec.node_selector = Some(node_selector.clone());
       }
       if let Some(ref affinity) = scheduling.node_affinity {
           pod_spec.affinity = Some(affinity.clone());
       }
   }
   ```

### Migration

**No migration needed**. Existing ConfigMaps without scheduling data:
- Are read successfully (fields default to None)
- Continue to work as before
- Can be updated to add scheduling configuration

### Testing Backward Compatibility

```rust
#[test]
fn test_resource_limit_without_scheduling() {
    let cm = ConfigMap {
        data: Some(btreemap! {
            "cpu".into() => "1000m".into(),
            "memory".into() => "512Mi".into(),
            // No scheduling fields
        }),
        // ...
    };
    
    let data = ResourceLimitData::try_from_config_map(cm).unwrap();
    assert!(data.scheduling.is_none());
}

#[test]
fn test_pod_creation_without_scheduling() {
    // Create resource limit without scheduling
    // Create pod from template
    // Verify pod has no nodeSelector or affinity
}
```

---

## 5. Integration Points Analysis

### Current Pod Creation Flow

```text
1. Client requests MCP server creation (via gRPC or HTTP)
   ↓
2. McpTemplateStore::create() called
   - Validates template exists
   - Retrieves ResourceLimit by name
   - Creates ConfigMap for MCP template
   ↓
3. PodMcpSessionManager::create_session() called
   - Generates session ID
   - Calls McpTemplateData::to_pod()
   ↓
4. McpTemplateData::to_pod() [KEY INTEGRATION POINT]
   - Retrieves ResourceLimit
   - Converts to ResourceRequirements (CPU, memory)
   - Builds Pod specification
   - Creates Pod via Kubernetes API
```

### File: store_mcp_template.rs, Method: to_pod()

**Current Code** (lines 161-256):
```rust
pub async fn to_pod(&self, session_id: &SessionId, client: &KubeStore) -> Result<Pod, AppError> {
    // Get resource limit
    let resource_limit = resource_limit_store.get(&self.resource_limit_name).await?;
    
    // ... env var setup ...
    
    let requirement = resource_limit.to_resource_requirements(); // Line 221
    
    Ok(Pod {
        metadata: ObjectMeta { /* ... */ },
        spec: Some(PodSpec {
            containers: vec![Container {
                resources: Some(requirement), // Resource requirements applied here
                // ...
            }],
            ..Default::default()  // ← Node selector and affinity go here!
        }),
        // ...
    })
}
```

**Required Change**:
```rust
let mut pod_spec = PodSpec {
    containers: vec![Container {
        resources: Some(requirement),
        // ...
    }],
    ..Default::default()
};

// NEW: Apply scheduling configuration
if let Some(scheduling) = resource_limit.scheduling {
    if let Some(node_selector) = scheduling.node_selector {
        pod_spec.node_selector = Some(node_selector);
    }
    if let Some(affinity) = scheduling.node_affinity {
        pod_spec.affinity = Some(affinity);
    }
}

Ok(Pod {
    metadata: /* ... */,
    spec: Some(pod_spec),
    // ...
})
```

### ResourceRequirements Conversion Pattern

**Current Method** (store_resource_limit.rs:92-136):
```rust
pub fn to_resource_requirements(&self) -> ResourceRequirements {
    ResourceRequirements {
        requests: Some(/* CPU, memory, ephemeral storage */),
        limits: Some(/* CPU limits, memory limits */),
        ..Default::default()
    }
}
```

**New Method Needed**:
```rust
pub fn to_scheduling_config(&self) -> Option<SchedulingConfig> {
    self.scheduling.clone()
}
```

### All ResourceLimit Read Paths

1. **store_resource_limit.rs**:
   - `get()` → Returns `Option<ResourceLimitData>`
   - `list()` → Returns `Vec<ResourceLimitData>`
   - `try_from_config_map()` → Parses ConfigMap to ResourceLimitData [MODIFY HERE]

2. **grpc/resource_limit.rs**:
   - `create_resource_limit()` → Accepts gRPC request [MODIFY HERE]
   - `get_resource_limit()` → Returns gRPC response [MODIFY HERE]
   - `list_resource_limits()` → Returns list response [MODIFY HERE]

3. **http/mcp/post_namespace_name.rs**:
   - HTTP endpoint for resource limit creation [MODIFY HERE]

4. **store_mcp_template.rs**:
   - `to_pod()` → Consumes ResourceLimit for pod creation [MODIFY HERE]

5. **Frontend**:
   - `models/resource_limit.rs` → Frontend model [MODIFY HERE]
   - `pages/resource_limits/create.rs` → Creation form [MODIFY HERE]
   - `pages/resource_limits/detail.rs` → Display view [MODIFY HERE]

---

## 6. Best Practices for Kubernetes Scheduling

### NodeSelector Best Practices

1. **Use Standard Labels**: Prefer well-known Kubernetes labels
   - `kubernetes.io/hostname`
   - `topology.kubernetes.io/zone`
   - `topology.kubernetes.io/region`
   - `node.kubernetes.io/instance-type`

2. **Custom Labels**: Use domain prefix for organization-specific labels
   - `company.io/gpu-type`
   - `company.io/disk-type`

3. **Simplicity**: NodeSelector is best for simple, hard requirements

### NodeAffinity Best Practices

1. **Required vs Preferred**:
   - **Required**: Hard constraint, pod won't schedule if not satisfied
   - **Preferred**: Soft constraint, scheduler tries to satisfy but will schedule anyway

2. **Operator Selection**:
   - `In`: Node label value must be in list
   - `NotIn`: Node label value must not be in list
   - `Exists`: Node must have the label (any value)
   - `DoesNotExist`: Node must not have the label
   - `Gt`/`Lt`: Numeric comparison (e.g., CPU count)

3. **Weight Strategy** (Preferred terms):
   - Higher weight (100) = stronger preference
   - Use weights to prioritize multiple preferences
   - Scheduler sums weights of satisfied preferences

4. **Combining NodeSelector and NodeAffinity**:
   - Both are applied (logical AND)
   - NodeSelector = simpler syntax for exact matches
   - NodeAffinity = complex logic with operators

### Example Use Cases

1. **GPU Nodes**:
   ```json
   {"nodeSelector": {"gpu": "true"}}
   ```

2. **Zone Spreading with Preference**:
   ```json
   {
     "nodeAffinity": {
       "requiredDuringSchedulingIgnoredDuringExecution": {
         "nodeSelectorTerms": [{
           "matchExpressions": [{
             "key": "topology.kubernetes.io/zone",
             "operator": "In",
             "values": ["us-west-1a", "us-west-1b", "us-west-1c"]
           }]
         }]
       },
       "preferredDuringSchedulingIgnoredDuringExecution": [{
         "weight": 100,
         "preference": {
           "matchExpressions": [{
             "key": "topology.kubernetes.io/zone",
             "operator": "In",
             "values": ["us-west-1a"]
           }]
         }
       }]
     }
   }
   ```
   This says: "Must be in us-west zones, prefer us-west-1a"

---

## 7. Frontend Considerations

### UI Component Strategy

1. **NodeSelector Input**:
   - Simple key-value pair form
   - Add/remove buttons for multiple entries
   - Validation on blur (label key/value format)

2. **NodeAffinity Input**:
   - Tabbed interface: "Required" and "Preferred"
   - Form builder for match expressions
   - Operator dropdown with help text
   - Values input (array of strings)
   - Weight slider for preferred terms (1-100)

3. **Display View**:
   - Collapsible sections for nodeSelector and nodeAffinity
   - Table format for nodeSelector
   - Nested list for nodeAffinity (terms → expressions)
   - Syntax-highlighted JSON view option

### Example Component Structure

```rust
// Simplified pseudo-code
fn NodeSelectorInput() -> Html {
    html! {
        <div class="node-selector">
            <h3>{"Node Selector"}</h3>
            <button onclick={add_selector_pair}>{"Add Label"}</button>
            {for selector_pairs.iter().map(|(key, value)| html! {
                <div class="selector-pair">
                    <input value={key} placeholder="key" />
                    <input value={value} placeholder="value" />
                    <button onclick={remove_pair}>{"Remove"}</button>
                </div>
            })}
        </div>
    }
}
```

---

## Summary of Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Protobuf Any handling | Store as JSON strings in ConfigMap | Simplicity, existing pattern, debuggability |
| Validation strategy | Two-tier: basic + Kubernetes API | Early feedback + future-proof |
| Backward compatibility | Optional fields at all layers | No breaking changes, zero migration |
| Storage format | JSON in ConfigMap data fields | Consistent with existing volumes field |
| Type safety | k8s-openapi types in Rust | Compile-time safety, full API compatibility |
| Frontend complexity | Simple key-value for nodeSelector, form builder for affinity | Matches usage frequency (nodeSelector more common) |

All research questions resolved. Ready for Phase 1: Data Model and Contract Design.
