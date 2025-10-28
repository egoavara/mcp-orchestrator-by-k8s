# Tasks: Resource Limit Scheduling Configuration

**Feature Branch**: `002-resource-limit-scheduling`  
**Input**: Design documents from `/specs/002-resource-limit-scheduling/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/resource_limit.proto

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and protobuf contract updates

- [X] T001 Update protobuf definition in spec/resource_limit.proto - add node_selector and node_affinity fields with google.protobuf.Any type
- [X] T002 Add import for google/protobuf/any.proto in spec/resource_limit.proto
- [X] T003 Regenerate protobuf Rust bindings by running cargo build in crates/proto/
- [X] T004 [P] Create validation module structure in crates/mcp-orchestrator/src/storage/scheduling_validation.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core scheduling infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T005 Add SchedulingConfig struct in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [X] T006 Add DATA_NODE_SELECTOR and DATA_NODE_AFFINITY constants in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [X] T007 [P] Implement validate_label_key function in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [X] T008 [P] Implement validate_label_value function in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [X] T009 [P] Implement validate_dns_subdomain function in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [X] T010 Implement validate_node_selector function in crates/mcp-orchestrator/src/storage/scheduling_validation.rs (depends on T007, T008)
- [X] T011 Implement validate_node_affinity function in crates/mcp-orchestrator/src/storage/scheduling_validation.rs (depends on T007, T008)
- [X] T012 [P] Implement SchedulingConfig::to_json_strings method in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [X] T013 [P] Implement SchedulingConfig::from_json_strings method in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [X] T014 Add mod scheduling_validation to crates/mcp-orchestrator/src/storage/mod.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Configure Node Selector for Resource Limits (Priority: P1) 🎯 MVP

**Goal**: Enable administrators to specify simple node selectors (key-value labels) for resource limits

**Independent Test**: Create a resource limit with nodeSelector {"gpu": "true"}, deploy an MCP server using that limit, and verify the pod is scheduled only on nodes with gpu=true label

### Implementation for User Story 1

- [X] T015 [US1] Update ResourceLimitData struct to add scheduling: Option<SchedulingConfig> field in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [X] T016 [US1] Modify ResourceLimitData::try_from_config_map to parse node_selector from ConfigMap data in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [X] T017 [US1] Modify ResourceLimitStore::create to serialize node_selector to ConfigMap data in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [X] T018 [US1] Update ResourceLimitData::to_resource_requirements to maintain backward compatibility in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [X] T019 [US1] Modify McpTemplateData::to_pod to extract nodeSelector from resource_limit.scheduling in crates/mcp-orchestrator/src/storage/store_mcp_template.rs (line ~240)
- [X] T020 [US1] Apply nodeSelector to PodSpec::node_selector in McpTemplateData::to_pod in crates/mcp-orchestrator/src/storage/store_mcp_template.rs
- [X] T021 [US1] Update create_resource_limit gRPC handler to extract nodeSelector from protobuf Any in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [X] T022 [US1] Call validate_node_selector in create_resource_limit gRPC handler in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [X] T023 [US1] Update get_resource_limit gRPC handler to serialize nodeSelector to protobuf Any in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [X] T024 [US1] Update list_resource_limits gRPC handler to include nodeSelector in responses in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [X] T025 [P] [US1] Add node_selector field to ResourceLimit model in crates/mcp-orchestrator-front/src/models/resource_limit.rs
- [X] T026 [P] [US1] Create NodeSelectorInput component for key-value pair entry in crates/mcp-orchestrator-front/src/components/node_selector_input.rs
- [X] T027 [US1] Integrate NodeSelectorInput into resource limit creation form in crates/mcp-orchestrator-front/src/pages/resource_limits/create.rs
- [X] T028 [US1] Display nodeSelector in resource limit detail view in crates/mcp-orchestrator-front/src/pages/resource_limits/detail.rs
- [ ] T029 [P] [US1] Write unit test for nodeSelector JSON serialization in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T030 [P] [US1] Write unit test for nodeSelector validation (valid labels) in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T031 [P] [US1] Write unit test for nodeSelector validation (invalid label key format) in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T032 [P] [US1] Write unit test for nodeSelector validation (invalid label value format) in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T033 [US1] Write integration test for creating resource limit with nodeSelector in tests/dependency_tests.rs
- [ ] T034 [US1] Write integration test for pod creation with nodeSelector applied in tests/dependency_tests.rs
- [ ] T035 [US1] Write integration test for backward compatibility (resource limit without nodeSelector) in tests/dependency_tests.rs

**Checkpoint**: At this point, User Story 1 should be fully functional - administrators can configure nodeSelector and pods are correctly scheduled

---

## Phase 4: User Story 2 - Configure Node Affinity for Resource Limits (Priority: P2)

**Goal**: Enable administrators to define complex node placement rules using node affinity (required/preferred rules with match expressions)

**Independent Test**: Create a resource limit with required nodeAffinity for zones ["us-west-1a", "us-west-1b"], deploy an MCP server, and verify the pod is scheduled only in those zones

### Implementation for User Story 2

- [ ] T036 [US2] Update ResourceLimitData::try_from_config_map to parse node_affinity from ConfigMap data in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T037 [US2] Update ResourceLimitStore::create to serialize node_affinity to ConfigMap data in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T038 [US2] Modify McpTemplateData::to_pod to extract nodeAffinity from resource_limit.scheduling in crates/mcp-orchestrator/src/storage/store_mcp_template.rs
- [ ] T039 [US2] Apply nodeAffinity to PodSpec::affinity in McpTemplateData::to_pod in crates/mcp-orchestrator/src/storage/store_mcp_template.rs
- [ ] T040 [P] [US2] Implement validate_node_selector_term in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T041 [P] [US2] Implement validate_node_selector_requirement in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T042 [US2] Complete validate_node_affinity implementation with required/preferred term validation in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T043 [US2] Update create_resource_limit gRPC handler to extract nodeAffinity from protobuf Any in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [ ] T044 [US2] Call validate_node_affinity in create_resource_limit gRPC handler in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [ ] T045 [US2] Update get_resource_limit gRPC handler to serialize nodeAffinity to protobuf Any in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [ ] T046 [US2] Update list_resource_limits gRPC handler to include nodeAffinity in responses in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [ ] T047 [P] [US2] Add node_affinity field and related types to ResourceLimit model in crates/mcp-orchestrator-front/src/models/resource_limit.rs
- [ ] T048 [P] [US2] Create NodeAffinityInput component for complex affinity rules in crates/mcp-orchestrator-front/src/components/node_affinity_input.rs
- [ ] T049 [US2] Integrate NodeAffinityInput into resource limit creation form in crates/mcp-orchestrator-front/src/pages/resource_limits/create.rs
- [ ] T050 [US2] Display nodeAffinity in resource limit detail view in crates/mcp-orchestrator-front/src/pages/resource_limits/detail.rs
- [ ] T051 [P] [US2] Write unit test for nodeAffinity JSON serialization in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T052 [P] [US2] Write unit test for nodeAffinity validation (valid required terms) in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T053 [P] [US2] Write unit test for nodeAffinity validation (valid preferred terms with weights) in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T054 [P] [US2] Write unit test for nodeAffinity validation (invalid operator) in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T055 [P] [US2] Write unit test for nodeAffinity validation (invalid weight range) in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T056 [P] [US2] Write unit test for nodeAffinity validation (operator-value mismatch) in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T057 [US2] Write integration test for creating resource limit with nodeAffinity in tests/dependency_tests.rs
- [ ] T058 [US2] Write integration test for pod creation with nodeAffinity applied in tests/dependency_tests.rs
- [ ] T059 [US2] Write integration test for combined nodeSelector and nodeAffinity in tests/dependency_tests.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently - administrators can use simple selectors OR complex affinity rules

---

## Phase 5: User Story 3 - Update Existing Resource Limits with Scheduling Configuration (Priority: P3)

**Goal**: Enable administrators to add or modify node selector and node affinity settings on existing resource limits

**Independent Test**: Update an existing resource limit to add nodeSelector, create a new MCP server using it, and verify new pods use the updated scheduling while existing pods remain unchanged

### Implementation for User Story 3

- [ ] T060 [US3] Implement ResourceLimitStore::update method for modifying resource limits in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T061 [US3] Add update_resource_limit gRPC service method definition in spec/service.proto
- [ ] T062 [US3] Implement update_resource_limit gRPC handler in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [ ] T063 [US3] Add validation in update handler to ensure scheduling changes don't affect existing pods in crates/mcp-orchestrator/src/grpc/resource_limit.rs
- [ ] T064 [P] [US3] Add UpdateResourceLimitRequest protobuf message in spec/resource_limit.proto
- [ ] T065 [P] [US3] Create resource limit edit page component in crates/mcp-orchestrator-front/src/pages/resource_limits/edit.rs
- [ ] T066 [US3] Integrate NodeSelectorInput and NodeAffinityInput into edit page in crates/mcp-orchestrator-front/src/pages/resource_limits/edit.rs
- [ ] T067 [US3] Add update API call in frontend API client in crates/mcp-orchestrator-front/src/api/resource_limits.rs
- [ ] T068 [P] [US3] Write unit test for ResourceLimitStore::update method in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T069 [US3] Write integration test for updating resource limit with new nodeSelector in tests/dependency_tests.rs
- [ ] T070 [US3] Write integration test for updating resource limit from nodeSelector to nodeAffinity in tests/dependency_tests.rs
- [ ] T071 [US3] Write integration test for removing scheduling configuration from resource limit in tests/dependency_tests.rs
- [ ] T072 [US3] Write integration test verifying existing pods unchanged after resource limit update in tests/dependency_tests.rs

**Checkpoint**: All user stories should now be independently functional - full CRUD operations on scheduling configuration

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T073 [P] Add comprehensive error messages for scheduling validation failures in crates/mcp-orchestrator/src/storage/scheduling_validation.rs
- [ ] T074 [P] Add logging for scheduling configuration operations in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T075 [P] Add logging for pod creation with scheduling in crates/mcp-orchestrator/src/storage/store_mcp_template.rs
- [ ] T076 [P] Update API documentation with scheduling examples in spec/README.md
- [ ] T077 [P] Add monitoring metrics for scheduling configuration usage in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T078 [P] Optimize JSON serialization performance for large affinity rules in crates/mcp-orchestrator/src/storage/store_resource_limit.rs
- [ ] T079 [P] Add frontend validation feedback for invalid label formats in crates/mcp-orchestrator-front/src/components/node_selector_input.rs
- [ ] T080 [P] Add frontend help text and examples for nodeAffinity operators in crates/mcp-orchestrator-front/src/components/node_affinity_input.rs
- [ ] T081 Run cargo fmt and cargo clippy on all modified files
- [ ] T082 Verify quickstart.md examples work end-to-end
- [ ] T083 Run full test suite with cargo test --workspace
- [ ] T084 [P] Add troubleshooting section to quickstart.md for common scheduling errors
- [ ] T085 [P] Create example ConfigMaps demonstrating scheduling configurations in specs/002-resource-limit-scheduling/examples/

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion (T001-T003) - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User Story 1 (P1): Can start immediately after Foundational
  - User Story 2 (P2): Can start after Foundational - independent of US1
  - User Story 3 (P3): Builds on US1 and US2 functionality but can be implemented independently
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
  - Delivers: Basic nodeSelector functionality for pod placement
  - MVP checkpoint: Fully functional simple node selection
  
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Independent of US1
  - Extends US1 with complex affinity rules
  - Can be developed in parallel with US1 if team capacity allows
  - Independent checkpoint: Affinity rules work with or without nodeSelector
  
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Logically follows US1+US2
  - Adds update capability for configurations created in US1 and US2
  - Can be deferred if update functionality not immediately needed
  - Workaround exists: Delete and recreate resource limits

### Within Each User Story

1. **Storage Layer**: Update ResourceLimitData struct → serialization methods → validation
2. **Pod Creation**: Modify to_pod method to apply scheduling configuration
3. **API Layer**: Update gRPC handlers to accept/return scheduling fields
4. **Frontend**: Update models → create input components → integrate into pages
5. **Tests**: Unit tests for serialization/validation → integration tests for end-to-end flows

### Parallel Opportunities

#### Phase 1 (Setup)
- T001-T002 (protobuf updates) must be sequential
- T003 (regenerate) depends on T001-T002
- T004 (create validation module) can run parallel with T003

#### Phase 2 (Foundational)
- T005-T006 (struct and constants) must complete first
- T007, T008, T009 (validation functions) can all run in parallel
- T010, T011 depend on T007-T009
- T012, T013 (SchedulingConfig methods) can run in parallel

#### User Story 1
- T025-T026 (frontend model and component) can run parallel with backend tasks T015-T024
- T029-T032 (unit tests) can all run in parallel
- T033-T035 (integration tests) can run in parallel after implementation complete

#### User Story 2
- T040-T041 (validation helpers) can run in parallel
- T047-T048 (frontend types and component) can run parallel with backend tasks
- T051-T056 (unit tests) can all run in parallel
- T057-T059 (integration tests) can run in parallel after implementation complete

#### User Story 3
- T064-T065 (protobuf and edit page) can run in parallel
- T068 (unit test) can run parallel with T069-T072 (integration tests)

---

## Parallel Execution Example: User Story 1

```bash
# Backend implementation (can run in parallel)
Task T015: "Update ResourceLimitData struct to add scheduling field"
Task T025: "Add node_selector field to frontend ResourceLimit model"
Task T026: "Create NodeSelectorInput component"

# After backend core complete, these can run in parallel
Task T029: "Write unit test for nodeSelector JSON serialization"
Task T030: "Write unit test for nodeSelector validation (valid labels)"
Task T031: "Write unit test for invalid label key format"
Task T032: "Write unit test for invalid label value format"

# After all implementation complete, integration tests in parallel
Task T033: "Integration test for creating resource limit with nodeSelector"
Task T034: "Integration test for pod creation with nodeSelector applied"
Task T035: "Integration test for backward compatibility"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

**Goal**: Basic node selection capability - most common use case

1. Complete Phase 1: Setup (T001-T004) - ~1-2 hours
2. Complete Phase 2: Foundational (T005-T014) - ~4-6 hours
3. Complete Phase 3: User Story 1 (T015-T035) - ~8-12 hours
4. **STOP and VALIDATE**: 
   - Create resource limit with nodeSelector {"region": "us-east"}
   - Deploy MCP server using that limit
   - Verify pod scheduled only on matching nodes
   - Test backward compatibility with existing resource limits
5. Deploy/demo if ready

**MVP Delivers**: 
- ✅ Simple node label-based scheduling
- ✅ gRPC and HTTP API support
- ✅ Frontend UI for nodeSelector
- ✅ Full validation
- ✅ Backward compatible

### Incremental Delivery

1. **Phase 1+2**: Setup + Foundational → ~6-8 hours
   - Foundation ready for all user stories
   
2. **Phase 3**: Add User Story 1 → ~8-12 hours
   - Test independently with nodeSelector configurations
   - Deploy/Demo (MVP!)
   - Covers 80% of use cases (simple label matching)
   
3. **Phase 4**: Add User Story 2 → ~10-14 hours
   - Test independently with nodeAffinity rules
   - Deploy/Demo with complex scheduling scenarios
   - Covers 95% of use cases (required + preferred rules)
   
4. **Phase 5**: Add User Story 3 → ~6-8 hours
   - Test update operations
   - Deploy/Demo with full CRUD capability
   - Covers 100% of requirements

5. **Phase 6**: Polish → ~4-6 hours
   - Documentation, optimization, monitoring
   - Production-ready quality

**Total Estimate**: 34-48 hours for full feature

### Parallel Team Strategy

With multiple developers:

1. **Team completes Setup + Foundational together** (~6-8 hours)
   
2. **Once Foundational is done:**
   - **Developer A**: User Story 1 (nodeSelector) - ~8-12 hours
   - **Developer B**: User Story 2 (nodeAffinity) - ~10-14 hours
   - **Developer C**: Polish tasks that don't block stories - ~4-6 hours
   
3. **After US1 and US2 complete:**
   - **Developer A or B**: User Story 3 (updates) - ~6-8 hours
   - **Developer C**: Remaining polish tasks
   
4. **Final integration and testing** - ~2-4 hours

**Parallel Team Estimate**: 20-28 hours elapsed time with 3 developers

---

## Verification Checklist

### User Story 1 Verification
- [ ] Can create resource limit with nodeSelector via gRPC
- [ ] Can create resource limit with nodeSelector via HTTP API
- [ ] Can view nodeSelector in resource limit details
- [ ] MCP server pods have nodeSelector in Pod spec
- [ ] Pods scheduled only on nodes with matching labels
- [ ] Existing resource limits without nodeSelector still work
- [ ] Invalid label formats rejected with clear error messages
- [ ] Frontend displays nodeSelector input form
- [ ] Frontend displays nodeSelector in detail view

### User Story 2 Verification
- [ ] Can create resource limit with required nodeAffinity
- [ ] Can create resource limit with preferred nodeAffinity
- [ ] Can combine required and preferred terms
- [ ] Can use all operators (In, NotIn, Exists, DoesNotExist, Gt, Lt)
- [ ] MCP server pods have affinity in Pod spec
- [ ] Invalid operators rejected
- [ ] Invalid weights rejected
- [ ] Operator-value mismatches rejected
- [ ] Can combine nodeSelector and nodeAffinity
- [ ] Frontend displays nodeAffinity input form
- [ ] Frontend displays complex affinity rules in detail view

### User Story 3 Verification
- [ ] Can update existing resource limit to add nodeSelector
- [ ] Can update existing resource limit to add nodeAffinity
- [ ] Can replace nodeSelector with nodeAffinity
- [ ] Can remove scheduling configuration
- [ ] New pods use updated scheduling configuration
- [ ] Existing pods retain original scheduling
- [ ] Update operation completes within 2 seconds
- [ ] Frontend edit page allows modification of scheduling

### Overall System Verification
- [ ] Backward compatibility: resource limits without scheduling work
- [ ] Performance: no degradation in pod creation time
- [ ] Validation: all edge cases caught before pod creation
- [ ] Error messages: clear and actionable
- [ ] Logging: scheduling operations logged appropriately
- [ ] Documentation: quickstart examples work end-to-end
- [ ] Tests: cargo test --workspace passes all tests

---

## Notes

- **[P] tasks**: Can run in parallel (different files, no dependencies)
- **[Story] label**: Maps task to specific user story for traceability
- **File paths**: All paths are absolute from repository root
- **Backward compatibility**: All changes maintain compatibility with existing resource limits
- **Testing strategy**: Unit tests for serialization/validation, integration tests for end-to-end flows
- **Error handling**: Two-tier validation (basic format in Rust, full validation by Kubernetes API)
- **JSON storage**: Consistent with existing volumes field pattern in ConfigMap
- **Type safety**: k8s-openapi types provide compile-time safety for pod creation

**Critical Success Factors**:
1. Complete Foundational phase before starting any user story
2. Test each user story independently before moving to next
3. Maintain backward compatibility throughout
4. Validate scheduling configuration before storage
5. Use existing patterns (JSON in ConfigMap) for consistency
