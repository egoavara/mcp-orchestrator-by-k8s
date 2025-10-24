use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::{Container, Pod, PodSpec};
use kube::{
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
    Api, Client,
};

use super::label_query::{build_label_selector, LabelQuery};
use super::labels::{
    add_prefix_to_user_labels, LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE, LABEL_TYPE_OF,
    TYPE_MCP_SERVER,
};
use crate::error::AppError;

pub struct McpServerStore {
    client: Client,
    default_namespace: String,
}

impl McpServerStore {
    pub fn new(client: Client, default_namespace: impl Into<String>) -> Self {
        Self {
            client,
            default_namespace: default_namespace.into(),
        }
    }

    fn api(&self, namespace: &str) -> Api<Pod> {
        Api::namespaced(self.client.clone(), namespace)
    }

    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
    }

    fn resolve_namespace(&self, namespace: Option<&str>) -> String {
        namespace.unwrap_or(&self.default_namespace).to_string()
    }

    pub async fn create(
        &self,
        namespace: &str,
        id: &str,
        labels: BTreeMap<String, String>,
        spec: PodSpec,
    ) -> Result<Pod, AppError> {
        let mut final_labels = add_prefix_to_user_labels(labels);
        final_labels.insert(
            LABEL_MANAGED_BY.to_string(),
            LABEL_MANAGED_BY_VALUE.to_string(),
        );
        final_labels.insert(LABEL_TYPE_OF.to_string(), TYPE_MCP_SERVER.to_string());

        let pod = Pod {
            metadata: ObjectMeta {
                name: Some(format!("mcp-server-{}", id)),
                namespace: Some(namespace.to_string()),
                labels: Some(final_labels),
                ..Default::default()
            },
            spec: Some(spec),
            ..Default::default()
        };

        self.api(namespace)
            .create(&PostParams::default(), &pod)
            .await
            .map_err(AppError::from)
    }

    pub async fn create_from_template(
        &self,
        namespace: &str,
        id: &str,
        template_labels: BTreeMap<String, String>,
        image: &str,
        command: Vec<String>,
        args: Vec<String>,
        env_vars: Vec<(String, String)>,
    ) -> Result<Pod, AppError> {
        let env = env_vars
            .into_iter()
            .map(|(name, value)| k8s_openapi::api::core::v1::EnvVar {
                name,
                value: Some(value),
                ..Default::default()
            })
            .collect();

        let spec = PodSpec {
            containers: vec![Container {
                name: "mcp-server".to_string(),
                image: Some(image.to_string()),
                command: if command.is_empty() {
                    None
                } else {
                    Some(command)
                },
                args: if args.is_empty() { None } else { Some(args) },
                env: Some(env),
                ..Default::default()
            }],
            restart_policy: Some("Never".to_string()),
            ..Default::default()
        };

        self.create(namespace, id, template_labels, spec).await
    }

    pub async fn get(&self, namespace: &str, id: &str) -> Result<Option<Pod>, AppError> {
        let name = format!("mcp-server-{}", id);
        match self.api(namespace).get(&name).await {
            Ok(pod) => {
                if let Some(labels) = &pod.metadata.labels {
                    if labels.get(LABEL_MANAGED_BY) != Some(&LABEL_MANAGED_BY_VALUE.to_string())
                        || labels.get(LABEL_TYPE_OF) != Some(&TYPE_MCP_SERVER.to_string())
                    {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
                Ok(Some(pod))
            }
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub async fn list(
        &self,
        namespace: Option<&str>,
        queries: &[LabelQuery],
    ) -> Result<Vec<Pod>, AppError> {
        let mut all_queries = vec![
            LabelQuery::Equal {
                key: LABEL_MANAGED_BY.to_string(),
                value: LABEL_MANAGED_BY_VALUE.to_string(),
            },
            LabelQuery::Equal {
                key: LABEL_TYPE_OF.to_string(),
                value: TYPE_MCP_SERVER.to_string(),
            },
        ];
        
        let prefixed_queries: Vec<LabelQuery> = queries
            .iter()
            .map(|q| q.clone().with_prefix())
            .collect();
        all_queries.extend(prefixed_queries);

        let selector = build_label_selector(&all_queries);
        let lp = ListParams::default().labels(&selector);

        let list = if let Some(ns) = namespace {
            self.api(ns).list(&lp).await.map_err(AppError::from)?
        } else {
            Api::<Pod>::all(self.client.clone())
                .list(&lp)
                .await
                .map_err(AppError::from)?
        };

        Ok(list.items)
    }

    pub async fn delete(&self, namespace: &str, id: &str) -> Result<(), AppError> {
        if self.get(namespace, id).await?.is_none() {
            return Err(AppError::NotFound(format!(
                "MCP Server {}/{} not found or not managed by mcp-orchestrator",
                namespace, id
            )));
        }

        let name = format!("mcp-server-{}", id);
        self.api(namespace)
            .delete(&name, &DeleteParams::default())
            .await
            .map_err(AppError::from)?;
        Ok(())
    }
}
