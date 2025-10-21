use crate::kube::pod_manager::PodManager;
use chrono::Utc;
use common::{
    failure::Failure,
    request::{CreateMcpPodRequest, ResourceRequirements},
    response::{McpPodResponse, PodPhase},
};
use k8s_openapi::api::core::v1::Pod;
use kube::Client;
use tracing::{error, info, instrument};

pub struct McpPodService {
    pod_manager: PodManager,
}

impl McpPodService {
    pub fn new(client: Client) -> Self {
        Self {
            pod_manager: PodManager::new(client),
        }
    }

    #[instrument(skip(self))]
    pub async fn create_pod(&self, req: CreateMcpPodRequest) -> Result<McpPodResponse, Failure> {
        if req.image.is_empty() {
            return Err(Failure::bad_request(
                "Image field is required and cannot be empty",
            ));
        }

        let env_vars: Vec<(String, String)> = req
            .env_vars
            .iter()
            .map(|e| (e.name.clone(), e.value.clone()))
            .collect();

        info!(image = %req.image, env_count = env_vars.len(), "Creating Pod");

        match self.pod_manager.create_pod(req).await {
            Ok((uuid, pod)) => {
                info!(pod_id = %uuid, "Pod created successfully");
                Ok(Self::pod_to_response(uuid.to_string(), pod))
            }
            Err(e) => {
                error!(error = ?e, "Failed to create Pod");
                Err(Failure::from_kube_error(e))
            }
        }
    }

    fn pod_to_response(id: String, pod: Pod) -> McpPodResponse {
        let metadata = pod.metadata;
        let spec = pod.spec.unwrap();
        let status = pod.status.unwrap_or_default();

        let phase = status
            .phase
            .as_deref()
            .map(PodPhase::from)
            .unwrap_or(PodPhase::Unknown);

        let created_at = metadata
            .creation_timestamp
            .map(|ts| ts.0)
            .unwrap_or_else(Utc::now);

        let container = &spec.containers[0];
        let image = container.image.clone().unwrap_or_default();

        let env_vars = container.env.as_ref().map(|envs| {
            envs.iter()
                .filter_map(|e| {
                    Some(common::request::EnvVar {
                        name: e.name.clone(),
                        value: e.value.clone()?,
                    })
                })
                .collect()
        });

        let resources = container.resources.as_ref().map(|r| ResourceRequirements {
            cpu: r
                .limits
                .as_ref()
                .and_then(|l| l.get("cpu"))
                .and_then(|q| q.0.parse().ok())
                .unwrap_or_else(|| "1".to_string()),
            memory: r
                .limits
                .as_ref()
                .and_then(|l| l.get("memory"))
                .and_then(|q| q.0.parse().ok())
                .unwrap_or_else(|| "1Gi".to_string()),
        });

        McpPodResponse {
            id,
            name: metadata.name.unwrap_or_default(),
            namespace: metadata
                .namespace
                .unwrap_or_else(|| "mcp-servers".to_string()),
            status: phase,
            image,
            created_at,
            labels: metadata.labels.unwrap_or_default(),
            env_vars,
            resources,
        }
    }
}
