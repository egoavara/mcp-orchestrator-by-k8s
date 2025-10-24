use std::collections::BTreeMap;

use proto::mcp::orchestrator::v1::{
    mcp_orchestrator_service_server::McpOrchestratorService, *,
};
use tonic::{Request, Response, Status};

use crate::error::AppError;
use crate::state::AppState;
use crate::storage::namespace_store::NamespaceStore;

pub struct GrpcService {
    state: AppState,
}

impl GrpcService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl McpOrchestratorService for GrpcService {
    async fn create_mcp_template(
        &self,
        _request: Request<CreateMcpTemplateRequest>,
    ) -> Result<Response<McpTemplateResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn get_mcp_template(
        &self,
        _request: Request<GetMcpTemplateRequest>,
    ) -> Result<Response<McpTemplateResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn list_mcp_templates(
        &self,
        _request: Request<ListMcpTemplatesRequest>,
    ) -> Result<Response<ListMcpTemplatesResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn delete_mcp_template(
        &self,
        _request: Request<DeleteMcpTemplateRequest>,
    ) -> Result<Response<DeleteMcpTemplateResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn get_mcp_server(
        &self,
        _request: Request<GetMcpServerRequest>,
    ) -> Result<Response<McpServerResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn list_mcp_servers(
        &self,
        _request: Request<ListMcpServersRequest>,
    ) -> Result<Response<ListMcpServersResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn create_namespace(
        &self,
        request: Request<CreateNamespaceRequest>,
    ) -> Result<Response<NamespaceResponse>, Status> {
        let req = request.into_inner();
        let store = NamespaceStore::new(self.state.kube_client.clone());

        let labels: BTreeMap<String, String> = req.labels.into_iter().collect();
        let annotations: BTreeMap<String, String> = req.annotations.into_iter().collect();

        let ns = store
            .create(&req.name, labels, annotations)
            .await
            .map_err(|e| Status::internal(format!("Failed to create namespace: {}", e)))?;

        let created_at = ns
            .metadata
            .creation_timestamp
            .as_ref()
            .and_then(|ts| ts.0.timestamp().try_into().ok())
            .unwrap_or(0);

        Ok(Response::new(NamespaceResponse {
            name: ns.metadata.name.unwrap_or_default(),
            labels: ns.metadata.labels.unwrap_or_default().into_iter().collect(),
            created_at,
            status: ns.status.as_ref().and_then(|s| s.phase.clone()).unwrap_or_default(),
        }))
    }

    async fn get_namespace(
        &self,
        request: Request<GetNamespaceRequest>,
    ) -> Result<Response<NamespaceResponse>, Status> {
        let req = request.into_inner();
        let store = NamespaceStore::new(self.state.kube_client.clone());

        let ns = store
            .get(&req.name)
            .await
            .map_err(|e| Status::internal(format!("Failed to get namespace: {}", e)))?
            .ok_or_else(|| Status::not_found(format!("Namespace {} not found", req.name)))?;

        let created_at = ns
            .metadata
            .creation_timestamp
            .as_ref()
            .and_then(|ts| ts.0.timestamp().try_into().ok())
            .unwrap_or(0);

        Ok(Response::new(NamespaceResponse {
            name: ns.metadata.name.unwrap_or_default(),
            labels: ns.metadata.labels.unwrap_or_default().into_iter().collect(),
            created_at,
            status: ns.status.as_ref().and_then(|s| s.phase.clone()).unwrap_or_default(),
        }))
    }

    async fn list_namespaces(
        &self,
        request: Request<ListNamespacesRequest>,
    ) -> Result<Response<ListNamespacesResponse>, Status> {
        let req = request.into_inner();
        let store = NamespaceStore::new(self.state.kube_client.clone());

        let namespaces = store
            .list(&[])
            .await
            .map_err(|e| Status::internal(format!("Failed to list namespaces: {}", e)))?;

        let responses: Vec<NamespaceResponse> = namespaces
            .into_iter()
            .map(|ns| {
                let created_at = ns
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .and_then(|ts| ts.0.timestamp().try_into().ok())
                    .unwrap_or(0);

                NamespaceResponse {
                    name: ns.metadata.name.unwrap_or_default(),
                    labels: ns.metadata.labels.unwrap_or_default().into_iter().collect(),
                    created_at,
                    status: ns.status.as_ref().and_then(|s| s.phase.clone()).unwrap_or_default(),
                }
            })
            .collect();

        let total = responses.len() as i32;

        Ok(Response::new(ListNamespacesResponse {
            namespaces: responses,
            next_page_token: String::new(),
            total_count: total,
        }))
    }

    async fn delete_namespace(
        &self,
        request: Request<DeleteNamespaceRequest>,
    ) -> Result<Response<DeleteNamespaceResponse>, Status> {
        use crate::storage::namespace_store::DeleteResult;
        
        let req = request.into_inner();
        let store = NamespaceStore::new(self.state.kube_client.clone());
        let pod_name = "manual-delete";

        let result = store
            .delete_with_lease(&req.name, pod_name, &self.state.default_namespace)
            .await
            .map_err(|e| match e {
                AppError::NotFound(msg) => Status::not_found(msg),
                _ => Status::internal(format!("Failed to delete namespace: {}", e)),
            })?;

        let (success, message) = match result {
            DeleteResult::Deleted => (true, format!("Namespace {} deleted successfully", req.name)),
            DeleteResult::Deleting => (true, format!("Namespace {} is being deleted", req.name)),
            DeleteResult::DeletionStarted(deps) => (false, format!(
                "Namespace {} deletion started but blocked by dependencies: {}",
                req.name, deps.join(", ")
            )),
            DeleteResult::FinalizerRemoved => (true, format!("Namespace {} finalizer removed, deletion will complete", req.name)),
            DeleteResult::HasDependencies(deps) => (false, format!(
                "Cannot delete namespace {}: it has dependencies: {}",
                req.name, deps.join(", ")
            )),
            DeleteResult::Locked => (false, format!(
                "Namespace {} is currently locked by another operation",
                req.name
            )),
        };

        if !success {
            return Err(Status::failed_precondition(message));
        }

        Ok(Response::new(DeleteNamespaceResponse {
            success,
            message,
        }))
    }

    async fn create_secret(
        &self,
        request: Request<CreateSecretRequest>,
    ) -> Result<Response<SecretResponse>, Status> {
        use crate::storage::secret_store::SecretStore;
        
        let req = request.into_inner();
        let store = SecretStore::new(
            self.state.kube_client.clone(),
            &req.namespace,
        );

        let namespace = if req.namespace.is_empty() {
            store.default_namespace()
        } else {
            &req.namespace
        };
        
        let labels: BTreeMap<String, String> = BTreeMap::new();
        let data: BTreeMap<String, Vec<u8>> = req.data.into_iter().collect();

        let secret = store
            .create(namespace, &req.name, labels, data, None)
            .await
            .map_err(|e| Status::internal(format!("Failed to create secret: {}", e)))?;

        let created_at = secret
            .metadata
            .creation_timestamp
            .as_ref()
            .and_then(|ts| ts.0.timestamp().try_into().ok())
            .unwrap_or(0);

        let keys: Vec<String> = secret.data.unwrap_or_default().keys().cloned().collect();

        Ok(Response::new(SecretResponse {
            name: secret.metadata.name.unwrap_or_default(),
            namespace: secret.metadata.namespace.unwrap_or_default(),
            r#type: req.r#type,
            created_at,
            updated_at: created_at,
            keys,
        }))
    }

    async fn get_secret(
        &self,
        request: Request<GetSecretRequest>,
    ) -> Result<Response<SecretResponse>, Status> {
        use crate::storage::secret_store::SecretStore;
        
        let req = request.into_inner();
        let store = SecretStore::new(
            self.state.kube_client.clone(),
            &req.namespace,
        );

        let namespace = if req.namespace.is_empty() {
            store.default_namespace()
        } else {
            &req.namespace
        };
        
        let secret = store
            .get(namespace, &req.name)
            .await
            .map_err(|e| Status::internal(format!("Failed to get secret: {}", e)))?
            .ok_or_else(|| Status::not_found(format!("Secret {}/{} not found", namespace, req.name)))?;

        let created_at = secret
            .metadata
            .creation_timestamp
            .as_ref()
            .and_then(|ts| ts.0.timestamp().try_into().ok())
            .unwrap_or(0);

        let keys: Vec<String> = secret.data.unwrap_or_default().keys().cloned().collect();

        Ok(Response::new(SecretResponse {
            name: secret.metadata.name.unwrap_or_default(),
            namespace: secret.metadata.namespace.unwrap_or_default(),
            r#type: 0,
            created_at,
            updated_at: created_at,
            keys,
        }))
    }

    async fn list_secrets(
        &self,
        request: Request<ListSecretsRequest>,
    ) -> Result<Response<ListSecretsResponse>, Status> {
        use crate::storage::secret_store::SecretStore;
        
        let req = request.into_inner();
        let default_ns = req.namespace.as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("default");
        
        let store = SecretStore::new(
            self.state.kube_client.clone(),
            default_ns,
        );

        let namespace = req.namespace.as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str());

        let secrets = store
            .list(namespace, &[])
            .await
            .map_err(|e| Status::internal(format!("Failed to list secrets: {}", e)))?;

        let responses: Vec<SecretResponse> = secrets
            .into_iter()
            .map(|secret| {
                let created_at = secret
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .and_then(|ts| ts.0.timestamp().try_into().ok())
                    .unwrap_or(0);

                let keys: Vec<String> = secret.data.unwrap_or_default().keys().cloned().collect();

                SecretResponse {
                    name: secret.metadata.name.unwrap_or_default(),
                    namespace: secret.metadata.namespace.unwrap_or_default(),
                    r#type: 0,
                    created_at,
                    updated_at: created_at,
                    keys,
                }
            })
            .collect();

        let total = responses.len() as i32;

        Ok(Response::new(ListSecretsResponse {
            secrets: responses,
            next_page_token: String::new(),
            total_count: total,
        }))
    }

    async fn update_secret(
        &self,
        _request: Request<UpdateSecretRequest>,
    ) -> Result<Response<SecretResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn delete_secret(
        &self,
        request: Request<DeleteSecretRequest>,
    ) -> Result<Response<DeleteSecretResponse>, Status> {
        use crate::storage::secret_store::SecretStore;
        use crate::storage::namespace_store::DeleteResult;
        
        let req = request.into_inner();
        let store = SecretStore::new(
            self.state.kube_client.clone(),
            &req.namespace,
        );

        let namespace = if req.namespace.is_empty() {
            store.default_namespace()
        } else {
            &req.namespace
        };
        
        let pod_name = "manual-delete";

        let result = store
            .delete_with_lease(namespace, &req.name, pod_name, &self.state.default_namespace)
            .await
            .map_err(|e| match e {
                AppError::NotFound(msg) => Status::not_found(msg),
                _ => Status::internal(format!("Failed to delete secret: {}", e)),
            })?;

        let (success, message) = match result {
            DeleteResult::Deleted => (true, format!("Secret {}/{} deleted successfully", namespace, req.name)),
            DeleteResult::Deleting => (true, format!("Secret {}/{} is being deleted", namespace, req.name)),
            DeleteResult::DeletionStarted(deps) => (false, format!(
                "Secret {}/{} deletion started but blocked by dependencies: {}",
                namespace, req.name, deps.join(", ")
            )),
            DeleteResult::FinalizerRemoved => (true, format!("Secret {}/{} finalizer removed, deletion will complete", namespace, req.name)),
            DeleteResult::HasDependencies(deps) => (false, format!(
                "Cannot delete secret {}/{}: it has dependencies: {}",
                namespace, req.name, deps.join(", ")
            )),
            DeleteResult::Locked => (false, format!(
                "Secret {}/{} is currently locked by another operation",
                namespace, req.name
            )),
        };

        if !success {
            return Err(Status::failed_precondition(message));
        }

        Ok(Response::new(DeleteSecretResponse {
            success,
            message,
        }))
    }

    async fn create_resource_limit(
        &self,
        _request: Request<CreateResourceLimitRequest>,
    ) -> Result<Response<ResourceLimitResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn get_resource_limit(
        &self,
        _request: Request<GetResourceLimitRequest>,
    ) -> Result<Response<ResourceLimitResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn list_resource_limits(
        &self,
        _request: Request<ListResourceLimitsRequest>,
    ) -> Result<Response<ListResourceLimitsResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }

    async fn delete_resource_limit(
        &self,
        _request: Request<DeleteResourceLimitRequest>,
    ) -> Result<Response<DeleteResourceLimitResponse>, Status> {
        Err(Status::unimplemented("not yet implemented"))
    }
}
