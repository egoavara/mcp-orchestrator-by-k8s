use common::request::CreateMcpPodRequest;
use k8s_openapi::api::core::v1::{
    Container, EnvVar as K8sEnvVar, Pod, PodSpec, ResourceRequirements as K8sResourceRequirements,
};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::PostParams;
use kube::{Api, Client};
use std::collections::BTreeMap;
use uuid::Uuid;

pub struct PodManager {
    client: Client,
    namespace: String,
}

impl PodManager {
    pub fn new(client: Client) -> Self {
        let namespace =
            std::env::var("KUBE_NAMESPACE").unwrap_or_else(|_| "mcp-servers".to_string());
        Self { client, namespace }
    }

    pub async fn create_pod(&self, req: CreateMcpPodRequest) -> Result<(Uuid, Pod), kube::Error> {
        let pod_id = Uuid::new_v4();
        let pod_name = format!("mcp-{}", &pod_id.to_string()[..8]);

        let mut labels = BTreeMap::new();
        labels.insert(
            "mcp-orchestrator.egoavara.net/kubernetes".to_string(),
            "1".to_string(),
        );
        labels.insert(
            "mcp-orchestrator.egoavara.net/pod-id".to_string(),
            pod_id.to_string(),
        );
        let resource = req.resources.unwrap_or_default();

        let env = req
            .env_vars
            .iter()
            .map(|v| K8sEnvVar {
                name: v.name.clone(),
                value: Some(v.value.clone()),
                ..Default::default()
            })
            .collect();

        let mut limits = BTreeMap::new();
        limits.insert("cpu".to_string(), Quantity(resource.cpu.clone()));
        limits.insert("memory".to_string(), Quantity(resource.memory.clone()));

        let mut requests = BTreeMap::new();
        requests.insert("cpu".to_string(), Quantity(resource.cpu.clone()));
        requests.insert("memory".to_string(), Quantity(resource.memory.clone()));

        let container = Container {
            name: "mcp-server".to_string(),
            image: Some(req.image),
            command: req.command,
            args: req.args,
            env: Some(env),
            stdin: Some(true),
            resources: Some(K8sResourceRequirements {
                limits: Some(limits),
                requests: Some(requests),
                ..Default::default()
            }),
            ..Default::default()
        };

        let pod = Pod {
            metadata: ObjectMeta {
                name: Some(pod_name),
                namespace: Some(self.namespace.clone()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(PodSpec {
                containers: vec![container],
                restart_policy: Some("Never".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let pods: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let created_pod = pods.create(&PostParams::default(), &pod).await?;

        Ok((pod_id, created_pod))
    }
}
