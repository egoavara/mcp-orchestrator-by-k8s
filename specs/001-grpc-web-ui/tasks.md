# Tasks: gRPC Service Management Web UI

**Input**: Design documents from `/specs/001-grpc-web-ui/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Not requested in specification - tests excluded from task list.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Frontend**: `crates/mcp-orchestrator-front/src/`
- **Shared Proto**: `crates/proto/src/`
- **Build Config**: `crates/mcp-orchestrator-front/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, build tools, and proto configuration

- [X] T001 Configure proto crate for WASM compatibility in crates/proto/Cargo.toml (remove transport feature, add only codegen and prost)
- [X] T002 [P] Add Trunk configuration file crates/mcp-orchestrator-front/Trunk.toml with dev server and proxy settings
- [X] T003 [P] Create index.html with WASM loader in crates/mcp-orchestrator-front/index.html
- [X] T004 [P] Create global styles.css in crates/mcp-orchestrator-front/styles.css
- [X] T005 Update frontend Cargo.toml dependencies (yew 0.21, yew-router 0.18, yewdux, tonic-web-wasm-client 0.8, gloo-net, serde)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T006 Initialize lib.rs with wasm-logger setup in crates/mcp-orchestrator-front/src/lib.rs
- [X] T007 Create main app component with router skeleton in crates/mcp-orchestrator-front/src/app.rs
- [X] T008 Define Route enum for all pages in crates/mcp-orchestrator-front/src/routes.rs
- [X] T009 [P] Create base gRPC client setup with URL configuration in crates/mcp-orchestrator-front/src/api/client.rs
- [X] T010 [P] Implement gRPC error mapping function in crates/mcp-orchestrator-front/src/api/client.rs
- [X] T011 [P] Create SessionState store (selected namespace, breadcrumbs) in crates/mcp-orchestrator-front/src/models/state.rs
- [X] T012 [P] Create UserPreferences store (theme, pagination) in crates/mcp-orchestrator-front/src/models/state.rs
- [X] T013 [P] Create FormValidation trait in crates/mcp-orchestrator-front/src/utils/validation.rs
- [X] T014 [P] Implement validation helpers (validate_name, validate_docker_image, validate_cpu, validate_memory) in crates/mcp-orchestrator-front/src/utils/validation.rs
- [X] T015 [P] Create date/time formatting utilities in crates/mcp-orchestrator-front/src/utils/format.rs
- [X] T016 [P] Create Layout component with navigation skeleton in crates/mcp-orchestrator-front/src/components/layout.rs
- [X] T017 [P] Create Navbar component in crates/mcp-orchestrator-front/src/components/navbar.rs
- [X] T018 [P] Create Loading spinner component in crates/mcp-orchestrator-front/src/components/loading.rs
- [X] T019 [P] Create ErrorMessage component in crates/mcp-orchestrator-front/src/components/error_message.rs
- [X] T020 [P] Create ConfirmDialog modal component in crates/mcp-orchestrator-front/src/components/confirm_dialog.rs
- [X] T021 [P] Create FormField input wrapper component in crates/mcp-orchestrator-front/src/components/form_field.rs
- [X] T022 [P] Create generic ResourceList component in crates/mcp-orchestrator-front/src/components/resource_list.rs
- [X] T023 [P] Create generic ResourceCard component in crates/mcp-orchestrator-front/src/components/resource_card.rs
- [X] T024 Create Home/Dashboard page in crates/mcp-orchestrator-front/src/pages/home.rs
- [X] T025 Wire up router in app.rs to show Home page and verify hot reload works

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - View and Manage MCP Templates (Priority: P1) 🎯 MVP

**Goal**: Enable administrators to list, create, view details, and delete MCP templates

**Independent Test**: Access templates page, view existing templates, create new template with name/image/command, verify it appears in list, view details, delete template

### Implementation for User Story 1

- [X] T026 [P] [US1] Create Template model wrapper in crates/mcp-orchestrator-front/src/models/template.rs
- [X] T027 [P] [US1] Implement templates API client (list, get, create, delete) in crates/mcp-orchestrator-front/src/api/templates.rs
- [X] T028 [US1] Create TemplateList page with state management in crates/mcp-orchestrator-front/src/pages/templates/list.rs
- [X] T029 [US1] Create TemplateDetail page in crates/mcp-orchestrator-front/src/pages/templates/detail.rs
- [X] T030 [US1] Create TemplateForm component with validation in crates/mcp-orchestrator-front/src/pages/templates/form.rs
- [X] T031 [US1] Wire up template routes in app.rs (list, detail, create paths)
- [X] T032 [US1] Add navigation links to templates in Navbar component
- [X] T033 [US1] Test complete template CRUD flow (create→list→detail→delete)

**Checkpoint**: User Story 1 (Templates) fully functional and independently testable

---

## Phase 4: User Story 2 - Manage Namespaces (Priority: P2)

**Goal**: Enable administrators to list, create, view details, and delete namespaces with resource dependency checks

**Independent Test**: Access namespaces page, create new namespace, view details showing contained resources, attempt to delete namespace with/without resources

### Implementation for User Story 2

- [X] T034 [P] [US2] Create Namespace model wrapper in crates/mcp-orchestrator-front/src/models/namespace.rs
- [X] T035 [P] [US2] Implement namespaces API client (list, get, create, delete) in crates/mcp-orchestrator-front/src/api/namespaces.rs
- [ ] T036 [P] [US2] Create NamespaceSelector dropdown component in crates/mcp-orchestrator-front/src/components/namespace_selector.rs
- [X] T037 [US2] Create NamespaceList page with state management in crates/mcp-orchestrator-front/src/pages/namespaces/list.rs
- [X] T038 [US2] Create NamespaceDetail page showing contained resources in crates/mcp-orchestrator-front/src/pages/namespaces/detail.rs
- [X] T039 [US2] Create NamespaceCreate form component in crates/mcp-orchestrator-front/src/pages/namespaces/create.rs
- [X] T040 [US2] Wire up namespace routes in app.rs (list, detail, create paths)
- [X] T041 [US2] Add navigation links to namespaces in Navbar component
- [ ] T042 [US2] Integrate NamespaceSelector into Layout for namespace-scoped resources
- [ ] T043 [US2] Add delete confirmation with resource dependency warning
- [ ] T044 [US2] Test complete namespace CRUD flow (create→list→detail→delete with validation)

**Checkpoint**: User Stories 1 AND 2 both work independently

---

## Phase 5: User Story 3 - Monitor MCP Servers (Priority: P3)

**Goal**: Enable administrators to view all running MCP server instances with status, namespace filtering, and navigation to related templates

**Independent Test**: Access servers page, view list of servers across namespaces, filter by namespace, check server status/details, click template name to navigate

### Implementation for User Story 3

- [ ] T045 [P] [US3] Create Server model wrapper in crates/mcp-orchestrator-front/src/models/server.rs
- [ ] T046 [P] [US3] Implement servers API client (list, get) in crates/mcp-orchestrator-front/src/api/servers.rs
- [ ] T047 [US3] Create ServerList page with namespace filter in crates/mcp-orchestrator-front/src/pages/servers/list.rs
- [ ] T048 [US3] Create ServerDetail page showing status, metrics, and template link in crates/mcp-orchestrator-front/src/pages/servers/detail.rs
- [ ] T049 [US3] Wire up server routes in app.rs (list, detail paths)
- [ ] T050 [US3] Add navigation links to servers in Navbar component
- [ ] T051 [US3] Add status badge component for server states (running/pending/failed) in ServerList
- [ ] T052 [US3] Implement namespace filtering logic with SessionState integration
- [ ] T053 [US3] Add clickable template name that navigates to TemplateDetail page
- [ ] T054 [US3] Test server monitoring flow (list→filter→detail→navigate to template)

**Checkpoint**: User Stories 1, 2, AND 3 all work independently

---

## Phase 6: User Story 4 - Manage Secrets (Priority: P4)

**Goal**: Enable administrators to securely create, view metadata, update, and delete secrets with key-value pairs (values masked for security)

**Independent Test**: Access secrets page in a namespace, create secret with key-value pairs, view secret showing only keys (not values), update secret, delete with warning if in use

### Implementation for User Story 4

- [X] T055 [P] [US4] Create Secret model wrapper in crates/mcp-orchestrator-front/src/models/secret.rs
- [X] T056 [P] [US4] Implement secrets API client (list, get, create, update, delete) in crates/mcp-orchestrator-front/src/api/secrets.rs
- [X] T057 [US4] Create SecretList page requiring namespace selection in crates/mcp-orchestrator-front/src/pages/secrets/list.rs
- [X] T058 [US4] Create SecretDetail page showing keys only (no values) in crates/mcp-orchestrator-front/src/pages/secrets/detail.rs
- [X] T059 [US4] Create SecretForm component with key-value input fields in crates/mcp-orchestrator-front/src/pages/secrets/create.rs
- [X] T060 [US4] Wire up secret routes in app.rs (list, detail, create, edit paths)
- [X] T061 [US4] Add navigation links to secrets in Navbar component
- [X] T062 [US4] Implement namespace requirement check (show error if no namespace selected)
- [ ] T063 [US4] Add update secret form with REPLACE/MERGE/PATCH strategy selection
- [X] T064 [US4] Add delete confirmation with warning about affected servers
- [X] T065 [US4] Ensure secret values are never exposed in UI or console (security validation)
- [X] T066 [US4] Test complete secret CRUD flow (create→list→detail→delete with validations)

**Checkpoint**: User Stories 1, 2, 3, AND 4 all work independently

---

## Phase 7: User Story 5 - Configure Resource Limits (Priority: P5)

**Goal**: Enable administrators to create, view, update, and delete resource limit configurations with CPU/memory constraints

**Independent Test**: Access resource limits page, create limit with CPU/memory values, view limits, update values, delete with warning about affected servers

### Implementation for User Story 5

- [X] T067 [P] [US5] Create ResourceLimit model wrapper in crates/mcp-orchestrator-front/src/models/resource_limit.rs
- [X] T068 [P] [US5] Implement resource limits API client (list, get, create, delete) in crates/mcp-orchestrator-front/src/api/resource_limits.rs
- [X] T069 [US5] Create ResourceLimitList page in crates/mcp-orchestrator-front/src/pages/resource_limits/list.rs
- [X] T070 [US5] Create ResourceLimitDetail page in crates/mcp-orchestrator-front/src/pages/resource_limits/detail.rs
- [X] T071 [US5] Create ResourceLimitForm component with CPU/memory validation in crates/mcp-orchestrator-front/src/pages/resource_limits/create.rs
- [X] T072 [US5] Wire up resource limit routes in app.rs (list, detail, create paths)
- [X] T073 [US5] Add navigation links to resource limits in Navbar component
- [X] T074 [US5] Implement CPU format validation (cores or millicores) in form
- [X] T075 [US5] Implement memory format validation (Ki/Mi/Gi/Ti) in form
- [X] T076 [US5] Add delete confirmation with affected servers warning
- [X] T077 [US5] Test complete resource limit CRUD flow (create→list→detail→delete with validations)

**Checkpoint**: All 5 user stories fully functional and independently testable

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T078 [P] Add pagination support to all list pages (templates, namespaces, servers, secrets, limits)
- [ ] T079 [P] Implement search/filter functionality on all list pages
- [ ] T080 [P] Add loading states to all async operations
- [ ] T081 [P] Add proper error handling and user-friendly error messages to all API calls
- [ ] T082 [P] Ensure responsive design for all pages (mobile, tablet, desktop)
- [ ] T083 [P] Add accessibility attributes (ARIA labels, keyboard navigation)
- [ ] T084 [P] Implement toast notifications for success/error feedback
- [ ] T085 [P] Add breadcrumb navigation using SessionState
- [ ] T086 [P] Optimize WASM bundle size (check <500KB target)
- [ ] T087 Code cleanup and refactoring (ensure all files <300 lines)
- [ ] T088 Documentation: Update README with development/build instructions from quickstart.md
- [ ] T089 Verify all success criteria from spec.md are met
- [ ] T090 Run full integration test following quickstart.md validation steps

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - User stories CAN proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3 → P4 → P5)
- **Polish (Phase 8)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1) - Templates**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2) - Namespaces**: Can start after Foundational (Phase 2) - Independent, but integrates with US1 via namespace selector
- **User Story 3 (P3) - Servers**: Can start after Foundational (Phase 2) - References US1 (template links) but independently testable
- **User Story 4 (P4) - Secrets**: Can start after Foundational (Phase 2) - Requires US2 (namespace selection) for context
- **User Story 5 (P5) - Resource Limits**: Can start after Foundational (Phase 2) - Independent, referenced by US1 (templates)

### Within Each User Story

- Models before API clients
- API clients before pages
- List pages before detail pages
- Detail pages before forms
- Core pages before integration with other stories
- Story complete before moving to next priority

### Parallel Opportunities

- **Phase 1 (Setup)**: T002, T003, T004 can run in parallel
- **Phase 2 (Foundational)**: T009-T023 can run in parallel (all different files)
- **Phase 3-7 (User Stories)**: 
  - Once Foundational complete, ALL user stories can start in parallel with different team members
  - Within each story: Models and API clients (marked [P]) can run in parallel
  - Example US1: T026 and T027 can run together
  - Example US2: T034, T035, T036 can run together
- **Phase 8 (Polish)**: T078-T086 can run in parallel (different concerns)

---

## Parallel Example: User Story 1 (Templates)

```bash
# Step 1: Launch model and API client together
Task T026: "Create Template model wrapper in src/models/template.rs"
Task T027: "Implement templates API client in src/api/templates.rs"

# Step 2: After both complete, build pages sequentially
Task T028: "Create TemplateList page" (depends on T026, T027)
Task T029: "Create TemplateDetail page" (depends on T026, T027)
Task T030: "Create TemplateForm component" (depends on T026, T027)
```

---

## Parallel Example: After Foundational Phase

With 5 developers, once Phase 2 completes:

```bash
Developer A → Phase 3: User Story 1 (Templates)
Developer B → Phase 4: User Story 2 (Namespaces)  
Developer C → Phase 5: User Story 3 (Servers)
Developer D → Phase 6: User Story 4 (Secrets)
Developer E → Phase 7: User Story 5 (Resource Limits)

All stories complete and integrate independently!
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup → Trunk configured, dependencies ready
2. Complete Phase 2: Foundational → Router, components, API client base ready
3. Complete Phase 3: User Story 1 (Templates) → T026-T033
4. **STOP and VALIDATE**: Test template CRUD independently
5. Build and deploy MVP (`trunk build --release`)

### Incremental Delivery

1. Setup + Foundational → Foundation ready (25 tasks)
2. Add US1 (Templates) → Test independently → Deploy MVP (8 tasks)
3. Add US2 (Namespaces) → Test independently → Deploy (11 tasks)
4. Add US3 (Servers) → Test independently → Deploy (10 tasks)
5. Add US4 (Secrets) → Test independently → Deploy (12 tasks)
6. Add US5 (Resource Limits) → Test independently → Deploy (11 tasks)
7. Polish all stories → Final release (13 tasks)

**Total: 90 tasks organized into 8 phases**

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup (Phase 1) together
2. Team completes Foundational (Phase 2) together - CRITICAL checkpoint
3. Once Foundational done, split team across user stories by priority:
   - Senior dev: US1 (Templates) - most critical
   - Mid dev: US2 (Namespaces) + US4 (Secrets) - related
   - Junior dev: US3 (Servers) - read-only, simpler
   - Another dev: US5 (Resource Limits) - independent
4. Stories complete in parallel, integrate cleanly
5. Team reconvenes for Polish phase

---

## Notes

- **[P] tasks**: Different files, no dependencies, safe to parallelize
- **[Story] label**: Maps task to specific user story for traceability
- **File size limit**: 300 lines per file (project guideline) - split if needed
- **Security**: T065 explicitly validates secret values never exposed
- **Testing**: Tests not requested in spec, excluded from task list
- **Each user story**: Independently completable and testable per spec requirements
- **MVP scope**: User Story 1 (Templates) delivers immediate value - 33 total tasks
- **Full feature**: All 5 user stories + polish - 90 total tasks
- **Commit strategy**: Commit after each task or logical group
- **Checkpoints**: Stop after each phase to validate story works independently
- **Success criteria**: Phase 8 (T089) validates all 8 success criteria from spec.md

---

## Task Summary

| Phase | Tasks | Parallelizable | Description |
|-------|-------|----------------|-------------|
| Phase 1: Setup | 5 | 3 | Proto config, Trunk, HTML, CSS, deps |
| Phase 2: Foundational | 20 | 18 | Router, components, API base, utils |
| Phase 3: US1 Templates (P1) 🎯 | 8 | 2 | MVP - Template CRUD |
| Phase 4: US2 Namespaces (P2) | 11 | 3 | Namespace CRUD + selector |
| Phase 5: US3 Servers (P3) | 10 | 2 | Server monitoring + status |
| Phase 6: US4 Secrets (P4) | 12 | 2 | Secret CRUD + security |
| Phase 7: US5 Resource Limits (P5) | 11 | 2 | Resource limit CRUD |
| Phase 8: Polish | 13 | 11 | Pagination, search, UX improvements |
| **TOTAL** | **90** | **43** | Complete gRPC Web UI |

**MVP (Phases 1-3)**: 33 tasks  
**Full Feature**: 90 tasks  
**Parallel opportunities**: 43 tasks marked [P] (48% of total)
