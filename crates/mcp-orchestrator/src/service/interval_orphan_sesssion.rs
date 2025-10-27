use std::collections::HashMap;

use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, ResourceExt, api::DeleteParams};

use crate::{
    state::AppState,
    storage::{
        annotations::ANNOTATION_LAST_ACCESS_AT, label_query::build_label_query,
        resource_type::RESOURCE_TYPE_MCP_SERVER,
    },
};

pub async fn check_orphan_session(state: &AppState) {
    tracing::debug!("Starting orphan MCP server pod check");
    let api = Api::<Pod>::all(state.kube_client.clone());
    let Ok(label_query) = build_label_query(RESOURCE_TYPE_MCP_SERVER, &[]) else {
        tracing::error!("Failed to build label query");
        return;
    };
    let now = Utc::now();
    let mut list_params = kube::api::ListParams::default()
        .labels(&label_query)
        .limit(32);
    let mut orphans = HashMap::<String, Vec<String>>::new();
    loop {
        let pods = match api.list(&list_params).await {
            Ok(pods) => pods,
            Err(e) => {
                tracing::error!("Failed to list MCP server pods: {}", e);
                return;
            }
        };

        for p in &pods.items {
            if is_pod_orphan(&now, p) {
                tracing::info!("Found orphan MCP server pod: {}", p.name_any());
                let namespace = p.namespace().unwrap_or_else(|| "default".to_string());
                let name = p.name_any();
                orphans.entry(namespace).or_default().push(name);
            }
        }

        if let Some(continue_token) = &pods.metadata.continue_ {
            tracing::info!(
                "MCP server pod list is paginated, remaining count is {}",
                pods.metadata.remaining_item_count.unwrap_or(0)
            );
            list_params = list_params.continue_token(continue_token);
        } else {
            break;
        }
    }

    for (namespace, pod_names) in orphans {
        let api = Api::<Pod>::namespaced(state.kube_client.clone(), &namespace);
        for pod_name in pod_names {
            match api.delete(&pod_name, &DeleteParams::default()).await {
                Ok(_) => {
                    tracing::info!("Deleted orphan MCP server pod: {}/{}", namespace, pod_name);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to delete orphan MCP server pod {}/{}: {}",
                        namespace,
                        pod_name,
                        e
                    );
                }
            }
        }
    }
}

fn is_pod_orphan(now: &DateTime<Utc>, pod: &Pod) -> bool {
    // Placeholder logic for determining if a pod is orphaned
    // In a real implementation, this would check for associated sessions or resources
    if let Some(creation_timestamp) = &pod.metadata.creation_timestamp {
        let creation_time = creation_timestamp.0.with_timezone(&Utc);
        let duration_since_creation = now.signed_duration_since(creation_time);
        if duration_since_creation.num_minutes() < 1 {
            tracing::debug!(
                "Pod {} is recently created ({}), not treating as orphan",
                pod.name_any(),
                duration_since_creation
            );
            return false;
        }
    }
    let Some(last_access_raw) = pod.annotations().get(ANNOTATION_LAST_ACCESS_AT) else {
        tracing::warn!(
            "Pod {} is missing last access annotation, treating as orphan",
            pod.name_any()
        );
        return true;
    };
    let Ok(last_access) = DateTime::parse_from_rfc3339(last_access_raw) else {
        tracing::warn!(
            "Pod {} has invalid last access label (value = {}), treating as orphan",
            pod.name_any(),
            last_access_raw
        );
        return true;
    };
    let last_access = last_access.with_timezone(&Utc);
    let duration_since_access = now.signed_duration_since(last_access);
    if duration_since_access.num_seconds() > 15 {
        return true;
    }
    false
}
