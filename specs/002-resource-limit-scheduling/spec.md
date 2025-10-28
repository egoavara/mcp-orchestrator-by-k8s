# Feature Specification: Resource Limit Scheduling Configuration

**Feature Branch**: `002-resource-limit-scheduling`  
**Created**: 2025-10-28  
**Status**: Draft  
**Input**: User description: "resource-limit 에 nodeselector, nodeaffinity 값을 받을 수 있게 만들기. protobuf 정의에 nodeselector 를 전부 정의하지 말고 any 타입으로 받아서 처리"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Configure Node Selector for Resource Limits (Priority: P1)

Administrators need to specify which Kubernetes nodes should run MCP server pods by setting node selector labels (e.g., region=us-west, gpu=true).

**Why this priority**: Essential for basic pod placement control. Allows administrators to target specific node groups by simple key-value matching, which is the most common scheduling use case.

**Independent Test**: Can be fully tested by creating a resource limit with nodeSelector, deploying an MCP server using that limit, and verifying the pod is scheduled only on nodes matching the selector labels.

**Acceptance Scenarios**:

1. **Given** a resource limit exists without node selector, **When** an administrator adds nodeSelector with "region=us-east", **Then** all future pods using this resource limit are scheduled only on nodes with the "region=us-east" label
2. **Given** a resource limit with nodeSelector "gpu=true", **When** an MCP server is created using this resource limit, **Then** the pod is scheduled only on nodes labeled with "gpu=true"
3. **Given** multiple resource limits with different node selectors, **When** administrators view the resource limit details, **Then** they can see the configured node selector for each resource limit

---

### User Story 2 - Configure Node Affinity for Resource Limits (Priority: P2)

Administrators need to define complex node placement rules using node affinity (required/preferred rules with multiple expressions) for more sophisticated scheduling scenarios.

**Why this priority**: Important for advanced use cases but not essential for initial deployment. Enables complex scheduling logic like "prefer GPU nodes but fallback to CPU nodes" or "must have SSD storage AND be in specific zones".

**Independent Test**: Can be tested independently by creating a resource limit with nodeAffinity rules, deploying an MCP server, and verifying the pod respects both required and preferred affinity rules.

**Acceptance Scenarios**:

1. **Given** a resource limit with required nodeAffinity for "topology.kubernetes.io/zone" in ["us-west-1a", "us-west-1b"], **When** an MCP server is created, **Then** the pod is only scheduled on nodes in those zones
2. **Given** a resource limit with preferred nodeAffinity for "node-type=gpu" with weight 100, **When** an MCP server is created and GPU nodes are available, **Then** the scheduler prefers GPU nodes but can fallback to other nodes
3. **Given** a resource limit with both required and preferred nodeAffinity rules, **When** an administrator views the resource limit, **Then** they can see all affinity rules with their operators and values

---

### User Story 3 - Update Existing Resource Limits with Scheduling Configuration (Priority: P3)

Administrators need to add or modify node selector and node affinity settings on existing resource limits without recreating them.

**Why this priority**: Convenience feature for operational flexibility. While useful, administrators can work around this by creating new resource limits if updates are not supported initially.

**Independent Test**: Can be tested by updating an existing resource limit's scheduling configuration and verifying that new MCP servers use the updated configuration while existing pods remain unchanged.

**Acceptance Scenarios**:

1. **Given** an existing resource limit without scheduling configuration, **When** an administrator adds nodeSelector, **Then** the resource limit is updated and new MCP servers use the new scheduling rules
2. **Given** a resource limit with nodeSelector, **When** an administrator replaces it with nodeAffinity rules, **Then** the resource limit reflects the new affinity configuration
3. **Given** a resource limit with scheduling configuration, **When** an administrator removes all scheduling rules, **Then** new MCP servers using this resource limit can be scheduled on any available node

---

### Edge Cases

- What happens when nodeSelector specifies labels that no nodes in the cluster have? (Pod remains in Pending state with clear error message)
- What happens when required nodeAffinity cannot be satisfied? (Pod remains in Pending state with scheduling failure event)
- What happens when both nodeSelector and nodeAffinity are specified? (Both must be satisfied - nodeSelector is logically ANDed with nodeAffinity required rules)
- What happens when nodeAffinity expressions conflict with each other? (Kubernetes scheduler validates compatibility; configuration should be rejected if invalid)
- What happens when updating a resource limit that is actively being used by running MCP servers? (Existing pods continue with old configuration, only new pods use updated configuration)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept nodeSelector as a flexible key-value map in resource limit configuration
- **FR-002**: System MUST accept nodeAffinity configuration including both required and preferred rules with multiple match expressions
- **FR-003**: System MUST store nodeSelector and nodeAffinity configurations using protobuf `google.protobuf.Any` type to maintain flexibility without full schema definition
- **FR-004**: System MUST apply nodeSelector to Kubernetes pod specifications when creating MCP server pods
- **FR-005**: System MUST apply nodeAffinity to Kubernetes pod specifications when creating MCP server pods
- **FR-006**: System MUST validate that nodeSelector contains valid Kubernetes label key-value pairs
- **FR-007**: System MUST validate that nodeAffinity contains valid Kubernetes affinity expression structure before accepting configuration
- **FR-008**: System MUST serialize and deserialize nodeSelector and nodeAffinity configurations correctly between protobuf, storage, and Kubernetes API formats
- **FR-009**: Administrators MUST be able to view nodeSelector and nodeAffinity settings when retrieving resource limit details
- **FR-010**: System MUST allow nodeSelector and nodeAffinity to be optional fields (not required for all resource limits)
- **FR-011**: System MUST support updating nodeSelector and nodeAffinity on existing resource limits
- **FR-012**: System MUST preserve existing resource limit behavior (CPU, memory, volumes) when scheduling configuration is added

### Key Entities

- **ResourceLimit**: Enhanced to include optional scheduling configuration (nodeSelector, nodeAffinity) in addition to existing resource constraints (CPU, memory, storage, volumes)
- **NodeSelector**: Flexible key-value map representing Kubernetes node label selectors
- **NodeAffinity**: Complex scheduling rules structure containing required/preferred node affinity terms with match expressions

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Administrators can configure node placement for MCP servers using simple label selectors with 100% success rate
- **SC-002**: Administrators can define complex scheduling rules using affinity expressions without system errors
- **SC-003**: MCP server pods are scheduled according to configured nodeSelector and nodeAffinity rules in 100% of cases where matching nodes exist
- **SC-004**: Resource limit creation with scheduling configuration completes within 2 seconds
- **SC-005**: System correctly rejects invalid scheduling configurations (malformed labels, invalid operators) with clear error messages
- **SC-006**: Existing MCP server functionality remains unaffected when scheduling configuration is not specified (backward compatibility maintained)
