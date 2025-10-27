# Data Models: gRPC Web UI

**Feature**: gRPC Service Management Web UI  
**Date**: 2025-10-27  
**Phase**: 1 - Design

## Overview

This document defines the data models for the frontend application. All models are derived from protobuf definitions in `spec/*.proto` and will be used as-is through the shared `proto` crate.

---

## Resource Scope Classification

### Cluster-Scoped Resources

Resources that exist independently at the cluster level, not bound to any namespace.

- **Namespace**: Top-level organizational container
- **ResourceLimit**: Cluster-wide resource policy configuration

### Namespace-Scoped Resources

Resources that must belong to a specific namespace and cannot exist independently.

- **McpTemplate**: MCP server configuration template
- **Secret**: Sensitive configuration data
- **McpServer**: Running MCP server instance (read-only from UI perspective)

---

## Entity Definitions

### 1. Namespace (Cluster-Scoped)

**Purpose**: Logical grouping for organizing MCP resources. Provides isolation between different projects or teams.

**Fields**:
- `name`: string (required, unique) - Namespace identifier
- `labels`: map<string, string> - Key-value labels for filtering
- `created_at`: string (ISO 8601) - Creation timestamp
- `deleted_at`: string | null (ISO 8601) - Soft deletion timestamp

**Validation Rules**:
- Name: lowercase alphanumeric + hyphens, max 63 characters
- Name must start/end with alphanumeric character
- Labels: keys follow Kubernetes label conventions

**State Transitions**:
```
[Created] → [Active] → [Deleting] → [Deleted]
```

**Relationships**:
- Has many: Templates, Secrets, Servers
- Deletion constraint: Must be empty (no child resources)

**Proto**: `namespace.proto` / `NamespaceResponse`

---

### 2. Resource Limit (Cluster-Scoped)

**Purpose**: Defines CPU and memory constraints for MCP server instances. Can be applied to specific namespaces or used as defaults.

**Fields**:
- `name`: string (required, unique) - Limit configuration identifier
- `description`: string - Human-readable description
- `limits`: ResourceLimit object - Resource constraints
  - `cpu`: string (required) - CPU request (e.g., "2", "500m")
  - `memory`: string (required) - Memory request (e.g., "4Gi", "512Mi")
  - `cpu_limit`: string | null - CPU limit (hard cap)
  - `memory_limit`: string | null - Memory limit (hard cap)
  - `ephemeral_storage`: string | null - Ephemeral storage limit
  - `volumes`: map<string, VolumeLimit> - Volume size limits
- `labels`: map<string, string> - Metadata labels
- `created_at`: string (ISO 8601)
- `deleted_at`: string | null (ISO 8601)

**Validation Rules**:
- CPU format: `<number>` (cores) or `<number>m` (millicores)
- Memory format: `<number>Ki|Mi|Gi|Ti`
- cpu_limit >= cpu (if both specified)
- memory_limit >= memory (if both specified)

**State Transitions**:
```
[Created] → [Active] → [Deleted]
```

**Relationships**:
- Referenced by: Templates
- Can be applied to: Namespaces (implicit through label queries)

**Proto**: `resource_limit.proto` / `ResourceLimitResponse`

---

### 3. MCP Template (Namespace-Scoped)

**Purpose**: Reusable configuration blueprint for creating MCP server instances. Defines Docker image, environment, and resource requirements.

**Fields**:
- `namespace`: string (required) - Parent namespace
- `name`: string (required) - Template identifier (unique within namespace)
- `labels`: map<string, string> - Metadata labels
- `image`: string (required) - Docker image name and tag
- `command`: string[] - Container entrypoint command override
- `args`: string[] - Command arguments
- `envs`: map<string, string> - Environment variables (plain text)
- `secret_envs`: string[] - Secret references for env vars
- `resource_limit_name`: string - Reference to ResourceLimit
- `volume_mounts`: VolumeMount[] - Volume mount configurations
- `secret_mounts`: SecretMount[] - Secret file mounts
- `created_at`: string (ISO 8601)
- `deleted_at`: string | null (ISO 8601)

**Validation Rules**:
- Name: alphanumeric + hyphens, max 63 characters
- Image: valid Docker image format (registry/repo:tag)
- secret_envs: must reference existing secrets in same namespace
- resource_limit_name: must reference existing ResourceLimit (if specified)
- secret_mounts: referenced secrets must exist in same namespace

**State Transitions**:
```
[Created] → [Active] → [Deleted]
```

**Relationships**:
- Belongs to: Namespace (required)
- References: ResourceLimit (optional), Secrets (via secret_envs, secret_mounts)
- Used by: MCP Servers

**Proto**: `mcp_template.proto` / `McpTemplateResponse`

**Nested Types**:
- `VolumeMount`: { pattern: string, mount_path: string }
- `SecretMount`: { name: string, mount_path: string }

---

### 4. Secret (Namespace-Scoped)

**Purpose**: Secure storage for sensitive configuration data (API keys, credentials, certificates).

**Fields**:
- `namespace`: string (required) - Parent namespace
- `name`: string (required) - Secret identifier (unique within namespace)
- `keys`: string[] - List of secret key names (values NOT included for security)
- `labels`: map<string, string> - Metadata labels
- `created_at`: string (ISO 8601)
- `deleted_at`: string | null (ISO 8601)

**Note**: Secret values are NEVER returned to the frontend. Only key names are exposed for UI display and validation.

**Create/Update Payload**:
- `data`: map<string, string> - Key-value pairs (values in plain text)
- Backend handles base64 encoding for Kubernetes storage

**Validation Rules**:
- Name: alphanumeric + hyphens, max 63 characters
- Keys: non-empty, alphanumeric + underscore
- Values: non-empty (enforced on create/update)

**State Transitions**:
```
[Created] → [Active] → [Updated] → [Deleted]
```

**Update Strategy** (enum):
- `REPLACE`: Replace all key-value pairs
- `MERGE`: Add new keys, update existing, keep others
- `PATCH`: Update only specified keys

**Relationships**:
- Belongs to: Namespace (required)
- Referenced by: Templates (via secret_envs, secret_mounts)

**Proto**: `secret.proto` / `SecretResponse`

---

### 5. MCP Server (Namespace-Scoped, Read-Only)

**Purpose**: Running instance of an MCP server based on a template. Managed by Kubernetes controller, read-only from UI perspective.

**Fields**:
- `namespace`: string - Parent namespace
- `name`: string - Server instance identifier
- `template_name`: string - Template used to create this server
- `status`: McpServerStatus enum - Current state
  - `PENDING`: Pod is being scheduled
  - `RUNNING`: Pod is running and ready
  - `FAILED`: Pod failed to start or crashed
  - `TERMINATED`: Pod was terminated
- `fqdn`: string - Fully qualified domain name for accessing server
- `host_ip`: string - Kubernetes node IP
- `pod_ip`: string[] - Pod internal IPs
- `created_at`: string (ISO 8601) - Pod creation time
- `ready_at`: string | null (ISO 8601) - Time when pod became ready
- `containers_ready_at`: string | null (ISO 8601) - Time when containers started

**State Transitions**:
```
[PENDING] → [RUNNING] → [TERMINATED]
            ↓
         [FAILED]
```

**Relationships**:
- Belongs to: Namespace
- Created from: Template
- Read-only: UI displays status, no create/update/delete operations

**Proto**: `mcp_server.proto` / `McpServerResponse`

**Extended Information** (from GetMcp call):
- `server_info`: ServerImplementation - MCP server metadata
- `capabilities`: ServerCapabilities - MCP protocol capabilities
- `instructions`: string | null - Usage instructions
- `tools`: Tool[] - Available MCP tools

---

## Common Types

### LabelQuery

**Purpose**: Filter resources by label selectors (Kubernetes-style).

**Fields**:
- `equal`: LabelKeyValue[] - Exact match (key = value)
- `not_equal`: LabelKeyValue[] - Exclusion (key != value)
- `in`: LabelKeyValues[] - Set membership (key in [values])
- `not_in`: LabelKeyValues[] - Set exclusion (key not in [values])
- `contain_key`: string[] - Key exists
- `not_contain_key`: string[] - Key does not exist

**Proto**: `common.proto` / `LabelQuery`

---

## UI-Specific Models

These models are used in the frontend for state management and are NOT derived from protobuf.

### SessionState (Yewdux Store)

**Purpose**: Track current UI session state with cross-tab synchronization.

```rust
#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "session", storage_tab_sync)]
struct SessionState {
    selected_namespace: Option<String>,  // Currently active namespace
    breadcrumbs: Vec<String>,            // Navigation path
    view_mode: ViewMode,                 // List/Grid display preference
}
```

### UserPreferences (Yewdux Store)

**Purpose**: Persistent user preferences across sessions.

```rust
#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
struct UserPreferences {
    theme: Theme,                        // Light/Dark theme
    items_per_page: usize,               // Pagination size
    default_namespace: Option<String>,   // Auto-select namespace on load
    show_deleted: bool,                  // Include soft-deleted resources
}
```

### ApiState<T> (Component State)

**Purpose**: Track async API call state with loading/error handling.

```rust
enum ApiState<T> {
    Idle,                               // No request made yet
    Loading,                            // Request in progress
    Success(T),                         // Data loaded successfully
    Error(String),                      // Error occurred with message
}
```

### FormState<T> (Component State)

**Purpose**: Manage form input with validation.

```rust
struct FormState<T> {
    data: T,                            // Form data
    errors: HashMap<String, String>,    // Field-level errors
    is_submitting: bool,                // Prevent double submission
    is_dirty: bool,                     // Track unsaved changes
}
```

---

## Validation Patterns

### Field Validators

```rust
fn validate_name(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("Name is required".to_string());
    }
    if name.len() > 63 {
        return Some("Name must be 63 characters or less".to_string());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Some("Name can only contain alphanumeric and hyphens".to_string());
    }
    if !name.chars().next().unwrap().is_alphanumeric() 
        || !name.chars().last().unwrap().is_alphanumeric() {
        return Some("Name must start and end with alphanumeric".to_string());
    }
    None
}

fn validate_docker_image(image: &str) -> Option<String> {
    if image.is_empty() {
        return Some("Docker image is required".to_string());
    }
    // Simple format check: [registry/]repo[:tag]
    let parts: Vec<&str> = image.split(':').collect();
    if parts.len() > 2 {
        return Some("Invalid image format".to_string());
    }
    None
}

fn validate_cpu(cpu: &str) -> Option<String> {
    // Accepts: "2", "500m", "0.5"
    let re = Regex::new(r"^\d+(\.\d+)?m?$").unwrap();
    if !re.is_match(cpu) {
        return Some("Invalid CPU format (e.g., '2', '500m')".to_string());
    }
    None
}

fn validate_memory(memory: &str) -> Option<String> {
    // Accepts: "512Mi", "4Gi", "1Ti"
    let re = Regex::new(r"^\d+(Ki|Mi|Gi|Ti)$").unwrap();
    if !re.is_match(memory) {
        return Some("Invalid memory format (e.g., '512Mi', '4Gi')".to_string());
    }
    None
}
```

---

## Data Flow

### Read Operations

```
User Action → Component → API Layer → gRPC-Web Client 
→ Backend gRPC Service → Kubernetes → Response → UI Update
```

### Write Operations

```
User Input → Form Validation → Submit → API Layer → gRPC-Web Client
→ Backend → Kubernetes Resource Created → Response → UI Feedback
```

### State Updates

```
API Response → Yewdux Store Update → Components Re-render
```

---

## Pagination Model

All list operations support cursor-based pagination:

**Request**:
- `first`: int | null - Number of items to return (default 20, max 100)
- `after`: string | null - Cursor for next page

**Response**:
- `data`: T[] - List of resources
- `endCursor`: string | null - Cursor for next page
- `hasNextPage`: bool - More data available

**UI Pattern**:
```rust
struct PaginationState {
    items: Vec<T>,
    cursor: Option<String>,
    has_more: bool,
    is_loading: bool,
}
```

---

## Summary

| Entity | Scope | Mutable | Key Relationships |
|--------|-------|---------|-------------------|
| **Namespace** | Cluster | Yes | Contains: Templates, Secrets, Servers |
| **ResourceLimit** | Cluster | Yes | Referenced by: Templates |
| **McpTemplate** | Namespace | Yes | Belongs to: Namespace; References: Secrets, ResourceLimit |
| **Secret** | Namespace | Yes | Belongs to: Namespace; Referenced by: Templates |
| **McpServer** | Namespace | No (read-only) | Belongs to: Namespace; Created from: Template |

All models use protobuf-generated types from the `proto` crate, ensuring type consistency between frontend and backend.
