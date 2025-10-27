# Research Findings: gRPC Web UI

**Feature**: gRPC Service Management Web UI  
**Date**: 2025-10-27  
**Phase**: 0 - Research

## Overview

This document consolidates research findings for building a Yew-based SPA that communicates with the gRPC backend service. All critical technical decisions are documented with rationale and alternatives considered.

---

## 1. gRPC-Web Client for WASM

### Decision

Use **`tonic-web-wasm-client` v0.8** for direct gRPC-Web calls from browser WASM to tonic backend.

### Rationale

1. **Backend Already Compatible**: The existing backend (main.rs:92) has `tonic_web::GrpcWebLayer` enabled, eliminating the need for a separate proxy (Envoy/grpcwebproxy)

2. **Type Safety**: Using protobuf with generated types provides compile-time type safety and consistency with the backend

3. **WASM-Native**: `tonic-web-wasm-client` is specifically designed for WASM environments using browser Fetch API

4. **Actively Maintained**: 135 stars, latest release v0.8.0 (August 2025), compatible with tonic 0.14+

5. **No Additional Infrastructure**: Direct browser → tonic communication without middleware

### Alternatives Considered

| Alternative | Why Rejected |
|------------|--------------|
| **Standard tonic client** | Requires tokio runtime features unavailable in WASM; transport layer incompatible with browsers |
| **JSON over REST API** | Would require building separate REST endpoints; lose type safety; double maintenance burden; proto definitions wasted |
| **Manual Fetch API** | Reinventing the wheel; complex protobuf encoding/decoding; error-prone |
| **gRPC-Web proxy (Envoy)** | Unnecessary since tonic-web handles gRPC-Web protocol natively; adds deployment complexity |

### Implementation Requirements

1. **Proto Crate Configuration**: Must disable `transport` feature, only use `codegen` and `prost`
   ```toml
   [dependencies]
   tonic = { version = "0.14", default-features = false, features = ["codegen", "prost"] }
   prost = "0.13"
   ```

2. **Frontend Dependencies**:
   ```toml
   tonic-web-wasm-client = "0.8"
   tonic = { version = "0.14", default-features = false, features = ["codegen", "prost"] }
   proto = { path = "../proto" }
   ```

3. **Client Setup**:
   ```rust
   use tonic_web_wasm_client::Client;
   
   let base_url = window().location().origin().unwrap();
   let client = Client::new(base_url);
   let mut grpc_client = McpOrchestratorServiceClient::new(client);
   ```

### Limitations

- **No Bidirectional Streaming**: gRPC-Web supports only:
  - ✅ Unary (request → response)
  - ✅ Server streaming (request → stream of responses)
  - ❌ Client streaming
  - ❌ Bidirectional streaming
  
  **Impact**: None for this project - all our operations are unary requests

- **CORS Required**: Backend must allow browser origins (already configured in main.rs:86-89)

- **Content-Type**: Requests use `application/grpc-web+proto` (handled automatically by tonic-web layer)

---

## 2. State Management

### Decision

Use **Yewdux** for global state management with built-in localStorage persistence.

### Rationale

1. **Global State Focus**: Designed specifically for app-wide state (selected namespace, user preferences)

2. **Built-in Persistence**: Native localStorage support via `#[store(storage = "local")]` macro - no manual implementation

3. **Tab Synchronization**: Automatic cross-tab state sync with `#[store(storage_tab_sync)]` attribute

4. **Less Boilerplate**: Simpler than Context API for global state scenarios

5. **Selective Re-rendering**: Components only re-render when their slice of state changes

6. **Context-Agnostic**: Can dispatch actions from anywhere (API layer, event handlers) without component tree access

### Alternatives Considered

| Alternative | Why Rejected |
|------------|--------------|
| **Yew Context API** | More boilerplate; requires ContextProvider wrapping; no built-in persistence; manual localStorage integration; all consumers re-render on any change |
| **Use `use_state` everywhere** | State not shared between pages; no persistence; duplicated state logic |
| **browser LocalStorage directly** | Manual serialization; no reactivity; error-prone sync logic; no type safety |

### Implementation Pattern

```rust
// Global preferences (persisted, not synced across tabs)
#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
struct UserPreferences {
    theme: Theme,
    items_per_page: usize,
    default_namespace: Option<String>,
}

// Session state (persisted, synced across tabs)
#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "session", storage_tab_sync)]
struct SessionState {
    selected_namespace: Option<String>,
    breadcrumbs: Vec<String>,
}

// Usage in component
#[function_component]
fn NamespaceSelector() -> Html {
    let (state, dispatch) = use_store::<SessionState>();
    
    let on_change = dispatch.reduce_mut_callback(|state| {
        state.selected_namespace = Some("production".to_string());
    });
    
    html! { /* ... */ }
}
```

### State Architecture

| State Type | Storage Mechanism | Use Case |
|------------|------------------|----------|
| **Global preferences** | Yewdux + localStorage | Theme, default namespace, pagination settings |
| **Session state** | Yewdux + sessionStorage | Currently selected namespace, navigation breadcrumbs |
| **Component state** | `use_state` hook | Form inputs, local UI toggles, temporary data |
| **API data** | `use_state` + async | Fetched resources, loading states, errors |

---

## 3. Form Validation

### Decision

Use **custom validation trait pattern** with error HashMap for client-side validation.

### Rationale

1. **Type Safety**: Rust trait system ensures all forms implement validation

2. **No External Dependencies**: Lightweight, WASM-compatible solution without adding dependencies

3. **Flexible**: Can implement domain-specific validation rules

4. **Real-time Feedback**: Validate on input change for immediate user feedback

### Alternatives Considered

| Alternative | Why Rejected |
|------------|--------------|
| **validator crate** | Heavy dependency; WASM compatibility uncertain; overkill for simple forms |
| **HTML5 native validation** | Limited control; poor UX; no custom error messages; browser-dependent |
| **Server-side only** | Poor UX; requires round-trip for simple validation; wastes bandwidth |

### Implementation Pattern

```rust
trait FormValidation {
    fn validate(&self) -> HashMap<String, String>;
    fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

#[derive(Default, Clone, PartialEq)]
struct CreateTemplateForm {
    name: String,
    image: String,
    namespace: String,
    errors: HashMap<String, String>,
}

impl FormValidation for CreateTemplateForm {
    fn validate(&self) -> HashMap<String, String> {
        let mut errors = HashMap::new();
        
        if self.name.is_empty() {
            errors.insert("name".into(), "Template name required".into());
        } else if !self.name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            errors.insert("name".into(), "Only alphanumeric and dash allowed".into());
        }
        
        if self.image.is_empty() {
            errors.insert("image".into(), "Docker image required".into());
        }
        
        errors
    }
}

// In component
let on_name_change = {
    let form = form.clone();
    Callback::from(move |e: Event| {
        let input = e.target_unchecked_into::<HtmlInputElement>();
        let mut new_form = (*form).clone();
        new_form.name = input.value();
        new_form.errors = new_form.validate();
        form.set(new_form);
    })
};
```

### Validation Rules by Resource

| Resource | Key Validations |
|----------|----------------|
| **Namespace** | Name: lowercase alphanumeric + hyphens, max 63 chars |
| **Template** | Name: required, alphanumeric + hyphens; Image: valid Docker image format |
| **Secret** | Name: required; Keys: non-empty; Values: minimum length (if applicable) |
| **Resource Limit** | CPU: valid format (e.g., "2", "500m"); Memory: valid format (e.g., "4Gi", "512Mi") |

---

## 4. Build and Development Workflow

### Decision

Use **Trunk** for building and serving the Yew application.

### Rationale

1. **Yew-Official**: Recommended build tool by Yew documentation

2. **Hot Reload**: Built-in hot module reloading for fast development

3. **Asset Pipeline**: Handles CSS, images, and other static assets automatically

4. **WASM Optimization**: Automatic wasm-opt integration for production builds

5. **Dev Server**: Built-in development server with proxy support

### Alternatives Considered

| Alternative | Why Rejected |
|------------|--------------|
| **wasm-pack** | Designed for npm publishing; no dev server; no asset pipeline; manual HTML setup |
| **Manual cargo + wasm-bindgen** | Too much boilerplate; no hot reload; error-prone |

### Setup

```toml
# Trunk.toml
[build]
target = "index.html"
release = false
dist = "dist"

[watch]
ignore = ["dist"]

[serve]
address = "127.0.0.1"
port = 8000

# Proxy API requests to backend
[[proxy]]
backend = "http://localhost:8080"
```

### Development Commands

```bash
# Install trunk
cargo install --locked trunk

# Development server with hot reload
trunk serve

# Production build
trunk build --release

# Clean build artifacts
trunk clean
```

### Integration with Workspace

The frontend crate (`mcp-orchestrator-front`) is already part of the cargo workspace. Trunk will:
1. Compile to WASM targeting `wasm32-unknown-unknown`
2. Generate `index.html` with script loading
3. Output to `dist/` directory
4. Serve on http://localhost:8000 (configurable)

---

## 5. Error Handling Patterns

### Decision

Map gRPC status codes to user-friendly messages with toast notifications for transient errors and inline errors for form validation.

### Rationale

1. **User-Friendly**: Technical gRPC errors translated to actionable messages

2. **Consistency**: Standardized error handling across all API calls

3. **Accessibility**: Visual feedback for errors with appropriate context

### Implementation Pattern

```rust
use tonic::Status;

fn map_grpc_error(status: Status) -> String {
    match status.code() {
        tonic::Code::NotFound => "Resource not found".to_string(),
        tonic::Code::AlreadyExists => "Resource already exists".to_string(),
        tonic::Code::PermissionDenied => "Permission denied".to_string(),
        tonic::Code::InvalidArgument => format!("Invalid input: {}", status.message()),
        tonic::Code::Unavailable => "Service temporarily unavailable, please retry".to_string(),
        _ => format!("Unexpected error: {}", status.message()),
    }
}

// Usage
async fn delete_resource(name: String) -> Result<(), String> {
    match grpc_client.delete_resource(request).await {
        Ok(_) => Ok(()),
        Err(status) => Err(map_grpc_error(status)),
    }
}
```

### Error Display Strategy

| Error Type | Display Method | Duration |
|------------|---------------|----------|
| **Transient (network, unavailable)** | Toast notification | 5 seconds, auto-dismiss |
| **Validation errors** | Inline under form field | Persistent until fixed |
| **Resource not found** | Page-level error message | Persistent with retry button |
| **Permission denied** | Modal dialog | Manual dismiss |

---

## 6. Protobuf Type Sharing

### Decision

Use shared `proto` crate with WASM-compatible configuration for both frontend and backend.

### Rationale

1. **Single Source of Truth**: Proto definitions in one place

2. **Type Consistency**: Frontend and backend use identical types

3. **Automatic Updates**: Proto changes propagate to both frontend and backend

4. **No Manual Conversion**: Direct use of generated types in UI

### Implementation

```toml
# crates/proto/Cargo.toml
[package]
name = "proto"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"

[dependencies]
tonic = { version = "0.14", default-features = false, features = ["codegen", "prost"] }
prost = "0.13"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
tonic-build = { version = "0.14", default-features = false, features = ["prost"] }
```

**Key**: No `transport` feature enables WASM compilation.

### Proto Generation

```rust
// crates/proto/build.rs
fn main() {
    tonic_build::configure()
        .build_server(false)  // Frontend doesn't need server traits
        .compile(
            &[
                "../../spec/service.proto",
                "../../spec/namespace.proto",
                "../../spec/mcp_template.proto",
                "../../spec/mcp_server.proto",
                "../../spec/secret.proto",
                "../../spec/resource_limit.proto",
            ],
            &["../../spec"],
        )
        .unwrap();
}
```

---

## Summary of Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| **UI Framework** | Yew | 0.21 |
| **Routing** | yew-router | 0.18 |
| **gRPC Client** | tonic-web-wasm-client | 0.8 |
| **State Management** | yewdux | Latest |
| **HTTP Client** | gloo-net | 0.4 |
| **Build Tool** | Trunk | Latest |
| **Protocol** | gRPC-Web + Protobuf | N/A |

---

## Next Steps (Phase 1)

1. ✅ Research complete
2. ⬜ Create data models (data-model.md)
3. ⬜ Define API contracts (contracts/)
4. ⬜ Write quickstart guide (quickstart.md)
5. ⬜ Update agent context

---

## References

- tonic-web-wasm-client: https://github.com/devashishdxt/tonic-web-wasm-client
- Yew documentation: https://yew.rs/docs/next/
- Yewdux: https://github.com/intendednull/yewdux
- Trunk: https://trunkrs.dev/
- gRPC-Web protocol: https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md
