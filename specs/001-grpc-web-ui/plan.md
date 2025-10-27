# Implementation Plan: gRPC Service Management Web UI

**Branch**: `001-grpc-web-ui` | **Date**: 2025-10-27 | **Spec**: [spec.md](./spec.md)  
**Input**: Feature specification from `/specs/001-grpc-web-ui/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Build a Single Page Application (SPA) using Rust Yew framework to manage all gRPC service resources (namespaces, templates, servers, secrets, resource limits) through a web browser interface. The UI will communicate with the existing gRPC backend service (defined in spec/service.proto) via gRPC-Web to enable administrators to create, view, update, and delete resources with appropriate validation and error handling.

## Technical Context

**Language/Version**: Rust 1.75+ (matching existing codebase), compiling to WebAssembly (WASM)  
**Primary Dependencies**: 
- Yew 0.21 (CSR mode) - UI framework
- yew-router 0.18 - client-side routing
- gloo-net 0.4 - HTTP/gRPC-Web client
- serde/serde_json - serialization
- tonic-web 0.14 - gRPC-Web protocol (needs confirmation for WASM compatibility)

**Storage**: Browser LocalStorage for UI state (selected namespace, preferences), no persistent backend storage in frontend  
**Testing**: wasm-pack test (WASM testing) and cargo test for non-WASM code  
**Target Platform**: Modern web browsers (Chrome, Firefox, Safari, Edge) via WebAssembly, requires JavaScript enabled  
**Project Type**: Web frontend (SPA) - adds to existing web application structure  
**Performance Goals**: 
- Initial page load <3 seconds
- UI interactions respond <500ms
- gRPC operations show loading state, complete within 2 seconds
- Handle lists of 100+ resources without performance degradation

**Constraints**: 
- Must use gRPC-Web protocol (gRPC is not directly callable from browsers)
- All secret values must be masked in UI (security requirement)
- Namespace-scoped resources (templates, secrets) require namespace selection
- Cluster-level resources (namespaces, resource limits) operate independently

**Scale/Scope**: 
- 5 main resource types to manage
- ~15-20 UI components
- Single admin UI (not multi-user with different views)
- Support namespaces with 100+ resources each

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Status**: ⚠️ NEEDS REVIEW - Constitution file is a template, no specific project rules defined

The constitution file (`.specify/memory/constitution.md`) contains only template placeholders. Since no specific project principles are defined, proceeding with standard web development best practices:

1. **Component-Based Architecture**: Yew components will be self-contained with clear props interfaces
2. **Type Safety**: Leverage Rust's type system for compile-time guarantees
3. **Error Handling**: All gRPC calls wrapped with proper error handling and user feedback
4. **Testing**: Unit tests for business logic, integration tests for API contracts
5. **Separation of Concerns**: UI components separate from API client logic

**No violations to report** - standard SPA architecture aligns with common best practices.

## Project Structure

### Documentation (this feature)

```text
specs/001-grpc-web-ui/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── api-types.md     # TypeScript/Rust type mappings for gRPC messages
│   └── routes.md        # Client-side routing specification
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/mcp-orchestrator-front/
├── src/
│   ├── lib.rs                      # Entry point, app initialization
│   ├── app.rs                      # Main app component with router
│   ├── api/                        # gRPC-Web client layer
│   │   ├── mod.rs
│   │   ├── client.rs               # Base gRPC client setup
│   │   ├── namespaces.rs           # Namespace API calls
│   │   ├── templates.rs            # Template API calls
│   │   ├── servers.rs              # Server API calls
│   │   ├── secrets.rs              # Secret API calls
│   │   └── resource_limits.rs     # Resource limit API calls
│   ├── components/                 # Reusable UI components
│   │   ├── mod.rs
│   │   ├── layout.rs               # Main layout with navigation
│   │   ├── navbar.rs               # Top navigation bar
│   │   ├── namespace_selector.rs  # Namespace dropdown selector
│   │   ├── resource_list.rs       # Generic list component
│   │   ├── resource_card.rs       # Generic card for resource items
│   │   ├── loading.rs              # Loading spinner
│   │   ├── error_message.rs       # Error display component
│   │   ├── confirm_dialog.rs      # Confirmation modal
│   │   └── form_field.rs           # Form input wrapper
│   ├── pages/                      # Page-level components
│   │   ├── mod.rs
│   │   ├── home.rs                 # Dashboard/home page
│   │   ├── namespaces/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs             # List namespaces
│   │   │   ├── detail.rs           # Namespace detail view
│   │   │   └── create.rs           # Create namespace form
│   │   ├── templates/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs             # List templates
│   │   │   ├── detail.rs           # Template detail view
│   │   │   └── form.rs             # Create/edit template form
│   │   ├── servers/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs             # List servers
│   │   │   └── detail.rs           # Server detail view
│   │   ├── secrets/
│   │   │   ├── mod.rs
│   │   │   ├── list.rs             # List secrets
│   │   │   ├── detail.rs           # Secret detail view
│   │   │   └── form.rs             # Create/edit secret form
│   │   └── resource_limits/
│   │       ├── mod.rs
│   │       ├── list.rs             # List resource limits
│   │       ├── detail.rs           # Resource limit detail view
│   │       └── form.rs             # Create/edit resource limit form
│   ├── models/                     # Data models (from protobuf)
│   │   ├── mod.rs
│   │   ├── namespace.rs            # Namespace models
│   │   ├── template.rs             # Template models
│   │   ├── server.rs               # Server models
│   │   ├── secret.rs               # Secret models
│   │   └── resource_limit.rs      # Resource limit models
│   ├── hooks/                      # Custom Yew hooks
│   │   ├── mod.rs
│   │   ├── use_api.rs              # Generic API call hook
│   │   └── use_namespace.rs       # Namespace context hook
│   ├── routes.rs                   # Route definitions
│   └── utils/                      # Utility functions
│       ├── mod.rs
│       ├── format.rs               # Date/time formatting
│       └── validation.rs           # Form validation helpers
├── dist/                           # Build output (gitignored)
├── index.html                      # HTML entry point
├── styles.css                      # Global styles
├── Cargo.toml                      # Package manifest
└── build.rs                        # Build script (if needed for protobuf)

crates/proto/
└── src/
    └── lib.rs                      # Generated protobuf types (shared with backend)

tests/
└── integration/
    └── ui_api_contract_test.rs     # Contract tests for gRPC-Web API
```

**Structure Decision**: 

The project follows a **Web Application** structure with separate frontend (mcp-orchestrator-front) and backend (mcp-orchestrator) crates. The frontend is a WASM-compiled Yew SPA that communicates with the existing gRPC backend.

**Key architectural decisions**:

1. **Shared Proto Crate**: The `crates/proto` crate contains protobuf-generated Rust types shared between frontend and backend, ensuring type consistency

2. **Component Hierarchy**:
   - `lib.rs` → `app.rs` (router) → `pages/` (routable views) → `components/` (reusable UI)
   
3. **API Layer Separation**: All gRPC-Web calls isolated in `api/` module for easier testing and mocking

4. **Page-per-Resource Pattern**: Each resource type gets its own page module with list/detail/form views

5. **Namespace Context**: Global state for currently selected namespace (for namespace-scoped resources)

6. **File Size Limit**: Following project guideline of 300 lines per file, complex pages/forms will be split into submodules

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations to report. Standard SPA architecture with component-based design fits within common web development patterns.

## Research Phase (Phase 0)

The following items require research and will be documented in `research.md`:

### 1. gRPC-Web Client for WASM

**Question**: How to call gRPC services from Yew/WASM? 

**Known constraints**:
- gRPC uses HTTP/2 which is not directly accessible from browsers
- Need gRPC-Web protocol (HTTP/1.1 or HTTP/2 with specific headers)
- Backend uses tonic-web 0.14

**Research needed**:
- Can tonic-generated client types work in WASM?
- Do we need a gRPC-Web proxy (envoy, grpcwebproxy)?
- Alternative: Convert gRPC to REST API for browser consumption?
- Libraries: tonic-web-wasm-client, grpc-web-rs, or custom fetch-based client?

### 2. Protobuf in WASM

**Question**: How to use protobuf-generated types in WASM frontend?

**Options to explore**:
- Use same prost-generated types as backend (shared crate)
- Generate TypeScript types and manually convert
- Use serde_json with JSON encoding instead of protobuf binary

### 3. State Management

**Question**: How to manage global state (selected namespace, auth, etc.)?

**Options to explore**:
- Yew Context API
- yewdux (Redux-like state management)
- Browser LocalStorage for persistence
- URL query parameters for selected namespace

### 4. Form Validation

**Question**: Client-side validation for complex forms (templates, secrets)?

**Options to explore**:
- validator crate (supports WASM)
- Custom validation with Rust functions
- HTML5 native validation

### 5. Error Handling Patterns

**Question**: How to surface gRPC errors to users in friendly way?

**Research needed**:
- Map tonic::Status codes to user messages
- Toast notifications vs inline errors
- Retry logic for transient failures

### 6. Build and Development Workflow

**Question**: How to develop and build the Yew SPA?

**Tooling needed**:
- trunk (Yew build tool) vs wasm-pack
- Hot reload during development
- Integration with existing cargo workspace
- Serving frontend alongside gRPC backend in production

## Design Phase (Phase 1)

To be completed in Phase 1 after research:

### Data Models (data-model.md)

Will extract entities from protobuf definitions:
- Namespace (cluster-scoped)
- ResourceLimit (cluster-scoped)
- McpTemplate (namespace-scoped)
- Secret (namespace-scoped)
- McpServer (namespace-scoped, read-only)

### API Contracts (contracts/)

Will document:
- gRPC-Web endpoint URLs
- Request/response types for each operation
- Error codes and handling
- Client-side route structure

### Quickstart (quickstart.md)

Will document:
- How to run frontend in development
- How to build for production
- How to configure gRPC backend URL
- Browser requirements

## Notes

**Critical Dependencies**:
1. Backend gRPC service must be running and accessible
2. gRPC-Web protocol support needed (may require proxy)
3. CORS configuration on backend for browser access

**Security Considerations**:
1. Secret values never sent to frontend (only keys returned)
2. All API calls over HTTPS in production
3. No authentication implemented yet (assumes network-level auth)

**User Experience Priorities**:
1. Clear loading states for all async operations
2. Validation feedback before submission
3. Confirmation for destructive operations
4. Namespace context preserved across navigation
