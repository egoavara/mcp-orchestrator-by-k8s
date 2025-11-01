<!--
  SYNC IMPACT REPORT
  ==================
  Version Change: [TEMPLATE] → 1.0.0
  
  Modified Principles:
    - [PRINCIPLE_1_NAME] → I. stdio-to-Streamable MCP Priority (NEW)
    - [PRINCIPLE_2_NAME] → II. Code Modularity & Size Discipline (NEW)
    - [PRINCIPLE_3_NAME] → III. Workspace-Level Dependency Management (NEW)
    - [PRINCIPLE_4_NAME] → IV. Kubernetes-Native Architecture (NEW)
    - [PRINCIPLE_5_NAME] → V. Build Discipline (NEW)
  
  Added Sections:
    - Core Principles (all 5 principles defined)
    - Technology Constraints
    - Development Workflow
    - Governance
  
  Removed Sections:
    - None (template placeholders replaced)
  
  Templates Requiring Updates:
    ✅ plan-template.md - Constitution Check section aligns with principles
    ✅ spec-template.md - Requirements section aligns with functional focus
    ✅ tasks-template.md - Task organization aligns with modularity principle
  
  Follow-up TODOs:
    - None; all placeholders filled with concrete project values
-->

# MCP Orchestrator Constitution

## Core Principles

### I. stdio-to-Streamable MCP Priority

**The primary value of this project is converting stdio-based MCP servers into streamable
(HTTP-SSE) MCP services.** Every architectural decision, feature prioritization, and
technical tradeoff MUST be evaluated against this core mission.

**Rationale**: stdio MCP servers cannot be used in web/cloud environments. This project
bridges that gap by wrapping them in Kubernetes pods and exposing them via HTTP
Server-Sent Events, enabling SaaS delivery of MCP capabilities.

**Rules**:
- Protocol bridge (stdio ↔ HTTP-SSE) MUST be the most reliable, performant component
- Features that do not directly support MCP orchestration require explicit justification
- Session management MUST maintain stdio semantics (stdin/stdout/stderr mapping)
- Breaking changes to the MCP protocol bridge require architecture review

### II. Code Modularity & Size Discipline

**All source files MUST remain under 300 lines where possible.** Code MUST be organized
into conceptually clear modules that can be understood in isolation.

**Rationale**: Large files become cognitive bottlenecks. Strict size limits force proper
separation of concerns and make the codebase navigable for new contributors and AI agents.

**Rules**:
- Files approaching 300 lines MUST be refactored into smaller modules
- Modules MUST have a single, clearly defined responsibility
- Cross-cutting concerns (logging, error handling) MUST be extracted into shared utilities
- Violations permitted only when splitting would harm conceptual clarity (document why)

### III. Workspace-Level Dependency Management

**Crate versions MUST be defined at the workspace level.** Sub-crates MUST reference
dependencies using `{workspace=true}`.

**Rationale**: Centralized version management prevents dependency conflicts, simplifies
auditing, and ensures consistency across all crates in the monorepo.

**Rules**:
- All dependency versions declared in root `Cargo.toml` `[workspace.dependencies]`
- Sub-crate `Cargo.toml` files use `dependency = { workspace = true }` syntax
- Version overrides in sub-crates require documented justification
- Similar libraries (e.g., `time` vs `chrono`) MUST NOT coexist; use existing choice

### IV. Kubernetes-Native Architecture

**The system MUST orchestrate MCP servers as Kubernetes Pods.** Every MCP server instance
runs in an isolated pod with defined resource limits and lifecycle management.

**Rationale**: Kubernetes provides battle-tested orchestration, scaling, isolation, and
resource control. Leveraging K8s primitives (Pods, ConfigMaps, Secrets) avoids reinventing
container management.

**Rules**:
- Each MCP server instance = one Kubernetes Pod
- Configuration via ConfigMaps (JSON data fields per 002-resource-limit-scheduling)
- Session state MUST survive pod restarts (use persistent storage or stateless design)
- Resource limits (CPU, memory) MUST be configurable per MCP server template

### V. Build Discipline

**Development work MUST use debug builds (`cargo build`). Release builds (`cargo build
--release`) are performed ONLY when explicitly requested by users.**

**Rationale**: Release builds are slow and unnecessary during iterative development.
This principle prevents wasted CI/CD cycles and developer time.

**Rules**:
- Default to `cargo build` and `cargo test` (debug mode)
- `cargo build --release` only on user request or production deployment
- `trunk serve` (frontend) is user-triggered only, not automatic
- CI pipelines MUST separate debug checks (fast feedback) from release builds (pre-deploy)

## Technology Constraints

**Language & Runtime**:
- Rust 1.90.0+ (as specified in `rust-toolchain.toml`)
- Backend: `axum` for HTTP API, `tonic` for gRPC
- Frontend: `trunk` + WebAssembly for admin UI

**Core Dependencies** (per Active Technologies in AGENTS.md):
- `kube` 0.x, `k8s-openapi` 0.x (Kubernetes client)
- `prost` (Protobuf), `tonic` (gRPC)
- `serde_json` (JSON serialization)
- See `Cargo.toml` workspace dependencies for authoritative list

**Storage**:
- Kubernetes ConfigMaps for resource limits and templates (JSON fields)
- Session metadata and MCP server state management TBD (evaluate stateless vs persistent)

**Communication Protocols**:
- HTTP + Server-Sent Events (SSE) for client ↔ MCP server streaming
- gRPC for internal service APIs (namespace, template, authorization management)
- stdio (stdin/stdout/stderr) for MCP server ↔ pod process communication

**Constraints**:
- MUST NOT introduce new time/date libraries if `chrono` or `time` already in use
- MUST NOT duplicate functionality covered by existing dependencies
- Configuration MUST be declarative (YAML/JSON) per `config.example.yaml`

## Development Workflow

**Planning & Specification**:
- Feature work begins with `/specs/###-feature-name/` documentation
- Use `.specify/templates/` for consistent spec, plan, and task structure
- Reference `specs/DEPENDENCY.md` for resource dependency graph

**Testing**:
- Unit tests: `cargo test` in respective crate
- Integration tests: `tests/` directories per crate
- Contract tests: validate gRPC/HTTP API contracts per protobuf definitions

**Code Review**:
- All changes via pull requests
- PR MUST reference spec document if implementing planned feature
- Breaking changes to MCP protocol bridge require architecture approval

**Deployment**:
- Backend: Docker image build via `Dockerfile`
- Frontend: `trunk build --release` (user-triggered)
- Kubernetes manifests in deployment scripts (TBD location)

## Governance

**Authority**: This constitution supersedes all other project practices and preferences.
When in conflict, constitution principles win.

**Amendments**:
- Amendments require documentation in this file with version bump
- Version follows semantic versioning:
  - MAJOR: Backward-incompatible governance changes, principle removal/redefinition
  - MINOR: New principle added, material expansion of existing guidance
  - PATCH: Clarifications, wording fixes, non-semantic refinements
- All amendments MUST include rationale and Sync Impact Report (HTML comment at top)

**Compliance**:
- All pull requests MUST verify alignment with Core Principles
- Violations of Principles I–V require explicit justification in PR description
- Complexity increases (new crates, new dependencies) MUST be documented in plan.md
  Complexity Tracking section
- Use `AGENTS.md` for runtime development guidance and technology tracking

**Review Cadence**:
- Constitution reviewed quarterly or when new major feature is planned
- Active Technologies section in `AGENTS.md` MUST be kept current with dependency changes

**Version**: 1.0.0 | **Ratified**: 2025-11-01 | **Last Amended**: 2025-11-01
