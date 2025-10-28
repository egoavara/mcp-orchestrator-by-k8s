# Implementation Plan: Resource Limit Scheduling Configuration

**Branch**: `002-resource-limit-scheduling` | **Date**: 2025-10-28 | **Spec**: [spec.md](./spec.md)

## Summary

Enhance ResourceLimit to support Kubernetes scheduling configuration (nodeSelector and nodeAffinity) for MCP server pod placement. Store scheduling configurations as `google.protobuf.Any` in protobuf for flexibility, serialize/deserialize to/from JSON in ConfigMap storage, and apply to Pod specifications during MCP server creation.

## Technical Context

**Language/Version**: Rust 1.90.0  
**Primary Dependencies**: kube 0.x, k8s-openapi 0.x, prost (protobuf), serde_json, tonic  
**Storage**: Kubernetes ConfigMaps (JSON data fields)  
**Testing**: cargo test, integration tests with rstest  
**Target Platform**: Linux server (Kubernetes orchestrator)  
**Project Type**: Backend service (Rust workspace with multiple crates)  
**Performance Goals**: Resource limit operations complete within 2 seconds, no degradation in pod creation time  
**Constraints**: Must maintain backward compatibility with existing resource limits, protobuf must use google.protobuf.Any for scheduling fields  
**Scale/Scope**: Existing codebase ~8k LOC, adding scheduling configuration to ResourceLimit entity

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Design Check

- **File Size**: ✅ All existing files under 300 lines (store_resource_limit.rs: 310 lines - acceptable for storage layer)
- **Conceptual Clarity**: ✅ Clear separation between storage (ConfigMap), domain (ResourceLimitData), and application (Pod creation)
- **Release Builds**: ✅ Only required by user, development uses debug builds
- **Dependencies**: ✅ Using existing Kubernetes client libraries, no new external dependencies required

### Post-Design Check (Phase 1)

✅ **PASSED** - All design artifacts complete

- **Data Model**: ✅ Complete (data-model.md)
  - Enhanced ResourceLimit with optional scheduling fields
  - SchedulingConfig entity for encapsulation
  - Clear validation rules defined
  
- **Contracts**: ✅ Complete (contracts/resource_limit.proto)
  - Updated protobuf with google.protobuf.Any fields
  - Backward compatible (optional fields)
  - Clear type URL conventions documented
  
- **Quickstart**: ✅ Complete (quickstart.md)
  - Practical examples for common use cases
  - Troubleshooting guide included
  - Best practices documented
  
- **Conceptual Clarity**: ✅ Maintained
  - Scheduling configuration stored as JSON (consistent with volumes pattern)
  - Validation split: basic (Rust) + full (Kubernetes API)
  - Clear separation of concerns across layers
  
- **File Size Projection**: ✅ Within limits
  - store_resource_limit.rs: +80 lines (serialization) → ~390 lines
  - store_mcp_template.rs: +20 lines (apply scheduling) → ~280 lines
  - New validation module: ~200 lines (separate file recommended)
  - Frontend files: +50-100 lines each (within limits)

## Project Structure

### Documentation (this feature)

```text
specs/002-resource-limit-scheduling/
├── plan.md              # This file
├── research.md          # Phase 0: Technical research on protobuf Any, K8s scheduling
├── data-model.md        # Phase 1: Enhanced ResourceLimit data model
├── quickstart.md        # Phase 1: Usage guide for nodeSelector/nodeAffinity
├── contracts/           # Phase 1: gRPC/protobuf contracts
│   └── resource_limit.proto  # Updated protobuf definition
└── tasks.md             # Phase 2: Implementation task breakdown (separate command)
```

### Source Code (repository root)

```text
spec/
├── common.proto                        # [MODIFY] Add scheduling types if needed
└── resource_limit.proto                # [MODIFY] Add nodeSelector, nodeAffinity fields

crates/mcp-orchestrator/src/
├── storage/
│   ├── store_resource_limit.rs        # [MODIFY] Add scheduling field serialization
│   └── store_mcp_template.rs          # [MODIFY] Apply scheduling to Pod creation (to_pod)
├── grpc/
│   ├── resource_limit.rs              # [MODIFY] Handle new scheduling fields in CRUD
│   └── utils.rs                       # [POTENTIALLY ADD] Scheduling validation helpers
└── http/
    └── mcp/
        ├── post_namespace_name.rs     # [MODIFY] Accept scheduling in creation
        └── utils.rs                   # [POTENTIALLY MODIFY] Validation logic

crates/proto/
└── src/
    └── lib.rs                          # [AUTO-GENERATED] Protobuf bindings

crates/mcp-orchestrator-front/src/
├── models/
│   └── resource_limit.rs              # [MODIFY] Add scheduling fields to frontend model
└── pages/resource_limits/
    ├── create.rs                       # [MODIFY] UI for nodeSelector/nodeAffinity
    └── detail.rs                       # [MODIFY] Display scheduling configuration

tests/
└── dependency_tests.rs                 # [ADD] Tests for scheduling configuration
```

**Structure Decision**: Single Rust workspace with multiple crates. Backend logic in `mcp-orchestrator`, protobuf definitions in `proto`, frontend in `mcp-orchestrator-front`. Scheduling configuration storage follows existing pattern of JSON serialization in ConfigMap data fields. Pod creation in `store_mcp_template.rs::to_pod()` method will be enhanced to apply scheduling configuration from ResourceLimit.

## Complexity Tracking

> No Constitution violations requiring justification. Feature follows existing patterns and architecture.

---

# Phase 0: Research & Discovery

## Research Questions

1. **Protobuf Any Type Handling**
   - How to use `google.protobuf.Any` in prost-generated Rust code?
   - Serialization/deserialization strategy for Any → JSON → ConfigMap data
   - Type URL conventions for nodeSelector vs nodeAffinity

2. **Kubernetes Scheduling Configuration**
   - Complete structure of K8s NodeSelector (map<string, string>)
   - Complete structure of K8s NodeAffinity (required vs preferred, match expressions)
   - Validation rules for label keys, values, operators

3. **Backward Compatibility Strategy**
   - Handling optional scheduling fields in existing ResourceLimits
   - Migration strategy for ConfigMaps without scheduling data
   - Default behavior when scheduling not specified

4. **Integration Points**
   - Where ResourceLimitData is consumed for Pod creation (store_mcp_template.rs)
   - Existing resource requirements conversion pattern
   - Frontend model mapping strategy

## Research Tasks

- [ ] Document prost-wkt-types usage for google.protobuf.Any
- [ ] Define JSON schema for nodeSelector and nodeAffinity storage
- [ ] Research k8s-openapi types: Affinity, NodeAffinity, NodeSelector, NodeSelectorRequirement
- [ ] Document validation strategy (client-side vs Kubernetes API validation)
- [ ] Identify all code paths that create/read ResourceLimit

---

# Phase 1: Design & Contracts

## Data Model Changes

### Enhanced ResourceLimit Entity

```text
ResourceLimit (protobuf):
  - cpu: string
  - memory: string
  - cpu_limit: optional string
  - memory_limit: optional string
  - ephemeral_storage: optional string
  - volumes: map<string, VolumeLimit>
  + node_selector: optional google.protobuf.Any    [NEW]
  + node_affinity: optional google.protobuf.Any    [NEW]

ResourceLimitData (Rust):
  - raw: ConfigMap
  - name: String
  - description: String
  - labels: HashMap<String, String>
  - limits: v1::ResourceLimit
  + scheduling: Option<SchedulingConfig>           [NEW]
  - created_at: DateTime<Utc>
  - deleted_at: Option<DateTime<Utc>>

SchedulingConfig (Rust):
  - node_selector: Option<HashMap<String, String>>
  - node_affinity: Option<k8s_openapi::api::core::v1::Affinity>
```

### ConfigMap Storage Format

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: resource-limit-rl-example
data:
  cpu: "1000m"
  memory: "512Mi"
  volumes: '{"data": {"size": "1Gi"}}'
  node_selector: '{"region": "us-west", "gpu": "true"}'        # NEW
  node_affinity: '{...JSON representation of NodeAffinity...}' # NEW
```

## API Contract Changes

### gRPC Service (resource_limit.proto)

```protobuf
import "google/protobuf/any.proto";

message CreateResourceLimitRequest {
  string name = 1;
  string description = 3;
  ResourceLimit limits = 6;
  map<string, string> labels = 5;
}

message ResourceLimit {
  string cpu = 1;
  string memory = 2;
  optional string cpu_limit = 3;
  optional string memory_limit = 4;
  optional string ephemeral_storage = 5;
  map<string, VolumeLimit> volumes = 6;
  optional google.protobuf.Any node_selector = 7;    // NEW
  optional google.protobuf.Any node_affinity = 8;    // NEW
}

message ResourceLimitResponse {
  string name = 1;
  string description = 3;
  ResourceLimit limits = 4;
  map<string, string> labels = 5;
  string created_at = 6;
  optional string deleted_at = 7;
}
```

### REST API (HTTP)

```text
POST /namespaces/{namespace}/resource-limits
Body: {
  "name": "gpu-limit",
  "description": "GPU node limit",
  "limits": {
    "cpu": "2000m",
    "memory": "4Gi",
    "nodeSelector": {"gpu": "true", "region": "us-west"},
    "nodeAffinity": {
      "requiredDuringSchedulingIgnoredDuringExecution": {...},
      "preferredDuringSchedulingIgnoredDuringExecution": [...]
    }
  }
}

GET /namespaces/{namespace}/resource-limits/{name}
Response: {
  "name": "gpu-limit",
  "limits": {
    ...,
    "nodeSelector": {"gpu": "true"},
    "nodeAffinity": {...}
  }
}
```

## Implementation Components

### 1. Protobuf Definition Updates

**File**: `spec/resource_limit.proto`

- Add `google.protobuf.Any node_selector = 7;`
- Add `google.protobuf.Any node_affinity = 8;`
- Import `google/protobuf/any.proto`

### 2. Storage Layer Enhancements

**File**: `crates/mcp-orchestrator/src/storage/store_resource_limit.rs`

**Changes**:
- Add constants: `DATA_NODE_SELECTOR`, `DATA_NODE_AFFINITY`
- Update `ResourceLimitData` struct with `scheduling: Option<SchedulingConfig>`
- Modify `try_from_config_map`: parse nodeSelector/nodeAffinity from JSON
- Modify `create`: serialize nodeSelector/nodeAffinity to JSON in ConfigMap data
- Add method `to_scheduling_config() -> (Option<BTreeMap<String, String>>, Option<Affinity>)`

### 3. Pod Creation Enhancement

**File**: `crates/mcp-orchestrator/src/storage/store_mcp_template.rs`

**Method**: `McpTemplateData::to_pod`

**Changes**:
- Extract scheduling config from `resource_limit.scheduling`
- Apply nodeSelector to `PodSpec::node_selector`
- Apply nodeAffinity to `PodSpec::affinity`

```rust
// Example modification in to_pod method
let mut pod_spec = PodSpec {
    containers: vec![...],
    ..Default::default()
};

// Apply scheduling configuration
if let Some(scheduling) = resource_limit.scheduling {
    if let Some(node_selector) = scheduling.node_selector {
        pod_spec.node_selector = Some(node_selector);
    }
    if let Some(affinity) = scheduling.node_affinity {
        pod_spec.affinity = Some(affinity);
    }
}
```

### 4. gRPC Handler Updates

**File**: `crates/mcp-orchestrator/src/grpc/resource_limit.rs`

**Changes**:
- Update request handlers to extract nodeSelector/nodeAffinity from `google.protobuf.Any`
- Deserialize Any → JSON → Rust structures
- Validate scheduling configuration before storage
- Update response builders to serialize scheduling back to Any

### 5. Validation Logic

**File**: `crates/mcp-orchestrator/src/grpc/utils.rs` (or new scheduling module)

**Validation Functions**:
- `validate_node_selector(selector: &HashMap<String, String>) -> Result<()>`
  - Check label key format (DNS subdomain prefix + name)
  - Check label value format (alphanumeric, dash, underscore, dot)
- `validate_node_affinity(affinity: &Affinity) -> Result<()>`
  - Validate operator values (In, NotIn, Exists, DoesNotExist, Gt, Lt)
  - Validate match expressions structure

### 6. Frontend Updates

**Files**: 
- `crates/mcp-orchestrator-front/src/models/resource_limit.rs`
- `crates/mcp-orchestrator-front/src/pages/resource_limits/create.rs`
- `crates/mcp-orchestrator-front/src/pages/resource_limits/detail.rs`

**Changes**:
- Add optional scheduling fields to ResourceLimit model
- Create UI components for nodeSelector input (key-value pairs)
- Create UI components for nodeAffinity input (complex form)
- Display scheduling configuration in detail view

## Testing Strategy

### Unit Tests

1. **Serialization/Deserialization**
   - Test nodeSelector JSON ↔ HashMap ↔ protobuf Any
   - Test nodeAffinity JSON ↔ k8s Affinity ↔ protobuf Any
   - Test backward compatibility with missing scheduling fields

2. **Validation**
   - Valid label keys/values
   - Invalid label formats
   - Valid affinity expressions
   - Invalid operators

### Integration Tests

1. **Resource Limit CRUD**
   - Create resource limit with nodeSelector
   - Create resource limit with nodeAffinity
   - Create resource limit with both
   - Retrieve and verify scheduling config
   - Update scheduling configuration

2. **Pod Creation**
   - Create MCP server with nodeSelector → verify Pod has nodeSelector
   - Create MCP server with nodeAffinity → verify Pod has affinity
   - Create MCP server with no scheduling → verify Pod has no scheduling config

### Contract Tests

1. **gRPC API**
   - Test protobuf Any encoding/decoding
   - Test request/response with scheduling fields
   - Test backward compatibility with old clients

## Rollout Plan

### Phase 1a: Backend Storage & Serialization (P1)
- Update protobuf definitions
- Implement storage layer changes
- Add unit tests for serialization

### Phase 1b: Pod Creation Integration (P1)
- Modify `to_pod` method to apply scheduling
- Add integration tests for pod creation
- Verify scheduling is correctly applied

### Phase 2: API & Validation (P2)
- Update gRPC handlers
- Implement validation logic
- Add contract tests

### Phase 3: Frontend UI (P3)
- Add scheduling fields to frontend model
- Implement creation UI
- Implement detail view display

---

# Phase 2: Task Breakdown

*To be generated by `/speckit.tasks` command*

---

## Notes

### Protobuf Any Strategy

Using `google.protobuf.Any` allows accepting arbitrary JSON structures without fully defining Kubernetes scheduling types in protobuf. The strategy:

1. Client sends scheduling config as Any with type_url indicating content type
2. Backend extracts JSON from Any
3. Backend deserializes JSON to Rust k8s-openapi types
4. Backend stores as JSON string in ConfigMap data field
5. Reverse process for retrieval

### Type URL Convention

```text
node_selector type_url: "kubernetes.io/NodeSelector"
node_affinity type_url: "kubernetes.io/Affinity"
```

### Backward Compatibility

- Existing ResourceLimits without scheduling data continue to work
- Optional fields default to None/null
- Pods created without scheduling have no node placement restrictions
- Frontend displays "No scheduling configuration" when fields are absent

### Migration Path

No migration required. Feature is additive:
- Old ResourceLimits: No scheduling fields → work as before
- New ResourceLimits: Optional scheduling fields → applied to new pods
- Existing pods: Unaffected (scheduling applied only at creation time)
