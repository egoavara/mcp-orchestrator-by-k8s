#!/bin/bash

# List services
grpcurl -plaintext 127.0.0.1:3000 list

# Namespace
grpcurl -plaintext -d '{"name":"test-ns","labels":{"env":"dev"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateNamespace
grpcurl -plaintext -d '{}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -d '{"label":{"equal":[{"key":"env","value":"dev"}]}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -d '{"label":{"not_contain_key":["aa"]}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -d '{"label":{"contain_key":["app.kubernetes.io/managed-by"]}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -d '{"label":{"contain_key":["env"]}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -d '{"name":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetNamespace
grpcurl -plaintext -d '{"name":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteNamespace
grpcurl -plaintext -d '{"name":"mcp-servers"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetNamespace

grpcurl -plaintext -d '{}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces

# Authorization
grpcurl -plaintext -d '{"name":"test-authz","namespace":"test-ns","type":"KUBERNETES_SERVICE_ACCOUNT"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateAuthorization
grpcurl -plaintext -d '{}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListAuthorizations
grpcurl -plaintext -d '{"namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListAuthorizations

# Generate Token
grpcurl -plaintext -d '{"name":"test-authz","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GenerateToken
grpcurl -plaintext -d '{"name":"test-authz","namespace":"test-ns","expire_duration":"600s"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GenerateToken

# Secret
grpcurl -plaintext -d '{"name":"test-secret00","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-b"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret01","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret02","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret03","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret04","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret05","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret06","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret07","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret08","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret09","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret10","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"name":"test-secret11","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -d '{"namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListSecrets
grpcurl -plaintext -d '{"namespace":"test-ns","first":2}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListSecrets
grpcurl -plaintext -d '{"name":"test-secret0","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetSecret
grpcurl -plaintext -d '{"name":"test-secret0","namespace":"test-ns","data":{"NEW_KEY":"bmV3dmFsdWU="}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/UpdateSecret
grpcurl -plaintext -d '{"name":"test-secret00","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteSecret
grpcurl -plaintext -d '{"name":"test-secret01","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteSecret

# ResourceLimit
grpcurl -plaintext -d '{"name":"m0-nano","limits":{"cpu":"150m","memory":"150Mi","volumes":{},"node_selector":{"beta.kubernetes.io/os":"linux"}},"labels":{},"description":"for general mcp server"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateResourceLimit
grpcurl -plaintext -d '{"name":"m0-small","limits":{"cpu":"250m","memory":"250Mi","volumes":{},"node_selector":{"beta.kubernetes.io/os":"linux"}},"labels":{},"description":"for general mcp server"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateResourceLimit
grpcurl -plaintext -d '{"name":"m0-medium","limits":{"cpu":"500m","memory":"500Mi","volumes":{},"node_selector":{"beta.kubernetes.io/os":"linux"}},"labels":{},"description":"for general mcp server"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateResourceLimit
grpcurl -plaintext -d '{"name":"m0-large","limits":{"cpu":"750m","memory":"750Mi","volumes":{},"node_selector":{"beta.kubernetes.io/os":"linux"}},"labels":{},"description":"for general mcp server"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateResourceLimit
grpcurl -plaintext -d '{"name":"m0-xlarge","limits":{"cpu":"1000m","memory":"1000Mi","volumes":{},"node_selector":{"beta.kubernetes.io/os":"linux"}},"labels":{},"description":"for general mcp server"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateResourceLimit
grpcurl -plaintext -d '{}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListResourceLimits
grpcurl -plaintext -d '{"name":"m0-small"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetResourceLimit
grpcurl -plaintext -d '{"name":"m0-nano"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteResourceLimit
grpcurl -plaintext -d '{"name":"m0-small"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteResourceLimit

# McpTemplate  optional string namespace = 1;
grpcurl -plaintext -d '{"name":"test-template0","namespace":"test-ns","resource_limit_name":"m0-medium","image":"node:25-alpine3.21","command":["npx"],"args":["--y","@modelcontextprotocol/server-memory"],"envs":{"LOG":"debug"},"secret_envs":["test-secret00"],"secret_mounts":[{"name":"test-secret01","mount_path":"/secret/test-secret01"}],"labels":{"app":"mcp"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateMcpTemplate
grpcurl -plaintext -d '{"name":"test-auth0","namespace":"test-ns","resource_limit_name":"m0-medium","authorization_name":"test-authz","image":"node:25-alpine3.21","command":["npx"],"args":["--y","@modelcontextprotocol/server-memory"],"envs":{"LOG":"debug"},"secret_envs":["test-secret00"],"secret_mounts":[{"name":"test-secret01","mount_path":"/secret/test-secret01"}],"labels":{"app":"mcp"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateMcpTemplate
grpcurl -plaintext -d '{"namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListMcpTemplates
grpcurl -plaintext -d '{"name":"test-template0","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetMcpTemplate
grpcurl -plaintext -d '{"name":"test-template0","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteMcpTemplate

# McpServer
grpcurl -plaintext -d '{"name":"test-server","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetMcpServer
grpcurl -plaintext -d '{"namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListMcpServers
