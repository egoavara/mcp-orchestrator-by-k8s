#!/bin/bash

# List services
grpcurl -plaintext -import-path ./spec -proto service.proto 127.0.0.1:3000 list

# Namespace
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-ns","labels":{"env":"dev"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateNamespace
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"label":{"equal":[{"key":"env","value":"dev"}]}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"label":{"not_contain_key":["aa"]}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"label":{"contain_key":["app.kubernetes.io/managed-by"]}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"label":{"contain_key":["env"]}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetNamespace
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteNamespace
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"mcp-servers"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetNamespace

# Secret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret00","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-b"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret01","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret02","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret03","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret04","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret05","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret06","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret07","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret08","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret09","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret10","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret11","namespace":"test-ns","data":{"API_KEY":"c2VjcmV0","TOKEN":"dG9rZW4="},"labels":{"type":"api-a"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListSecrets
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"namespace":"test-ns","first":2}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListSecrets
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret0","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret0","namespace":"test-ns","data":{"NEW_KEY":"bmV3dmFsdWU="}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/UpdateSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret00","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteSecret
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-secret01","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteSecret

# ResourceLimit
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"m0-nano","limits":{"cpu":"100m","memory":"128Mib","volumes":{}},"labels":{},"description":"for general mcp server"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateResourceLimit
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"m0-small","limits":{"cpu":"250m","memory":"256Mib","volumes":{}},"labels":{},"description":"for general mcp server"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateResourceLimit
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListResourceLimits
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"m0-small"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetResourceLimit
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"m0-small"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteResourceLimit

# McpTemplate  optional string namespace = 1;
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-template0","namespace":"test-ns","image":"25-alpine3.21","command":["npx"],"args":["--y","@modelcontextprotocol/server-memory"],"envs":{"LOG":"debug"},"secret_envs":["test-secret00"],"secret_mounts":[{"name":"test-secret01","mount_path":"/secret/test-secret01"}],"resource_limit_name":"m0-nano","labels":{"app":"mcp"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateMcpTemplate
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-template1","namespace":"test-ns","image":"test-image","command":["/bin/mcp"],"envs":{"LOG":"debug"},"resource_limit_name":"m0-nano","labels":{"app":"mcp"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateMcpTemplate
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-template2","namespace":"test-ns","image":"test-image","command":["/bin/mcp"],"args":["--port","8080"],"envs":{"LOG":"debug"},"resource_limit_name":"unknown","labels":{"app":"mcp"}}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/CreateMcpTemplate
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListMcpTemplates
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-template0","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetMcpTemplate
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-template0","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/DeleteMcpTemplate

# McpServer
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"name":"test-server","namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/GetMcpServer
grpcurl -plaintext -import-path ./spec -proto service.proto -d '{"namespace":"test-ns"}' 127.0.0.1:3000 mcp.orchestrator.v1.McpOrchestratorService/ListMcpServers
