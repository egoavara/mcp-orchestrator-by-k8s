use chrono::Duration;
use futures::TryStreamExt;
use k8s_openapi::api::core::v1::{ConfigMap, Namespace, Secret};
use kube::{
    Api, Resource, ResourceExt,
    runtime::{
        metadata_watcher,
        watcher::{Config, Event},
    },
};
use tokio_util::sync::CancellationToken;

mod interval_orphan_sesssion;
pub(crate) mod util;

use crate::{
    service::util::interval_handler,
    state::AppState,
    storage::{
        label_query::build_label_query,
        resource_type::{
            RESOURCE_TYPE_MCP_TEMPLATE, RESOURCE_TYPE_NAMESPACE, RESOURCE_TYPE_PREFIX_MCP_TEMPLATE,
            RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT, RESOURCE_TYPE_PREFIX_SECRET,
            RESOURCE_TYPE_RESOURCE_LIMIT, RESOURCE_TYPE_SECRET,
        },
        resource_uname::filter_relpath,
        store::KubeStore,
        util_delete::DeleteOption,
        util_name::{decode_k8sname, encode_k8sname},
    },
};

pub async fn listeners(state: AppState, ct: CancellationToken) {
    // let pod = Api::<Pod>::all(state.kube_client.clone());
    // let secret = Api::<Secret>::all(state.kube_client.clone());
    tokio::spawn(namespace_listener(state.clone()));
    tokio::spawn(secret_listener(state.clone()));
    tokio::spawn(mcp_template_listener(state.clone()));
    tokio::spawn(resource_limit_listener(state.clone()));
    interval_handler(
        state,
        Duration::seconds(15),
        ct.clone(),
        crate::make_interval_handler!(interval_orphan_sesssion::check_orphan_session),
    );
}

async fn secret_listener(state: AppState) {
    let secret = Api::<Secret>::all(state.kube_client.clone());
    let label = build_label_query(RESOURCE_TYPE_SECRET, &[]).unwrap();
    let watch = metadata_watcher(secret, Config::default().labels(&label).timeout(30));
    let mut watch = Box::pin(watch);
    while let Some(event) = watch.try_next().await.unwrap() {
        match event {
            Event::Apply(data) | Event::InitApply(data) | Event::Delete(data) => {
                let namespace = data.namespace().unwrap_or_else(|| "default".to_string());
                let raw_name = data.name_any();
                tokio::spawn(handle_delete_secret(
                    state.kube_store.clone(),
                    namespace,
                    raw_name,
                ));
            }
            Event::Init | Event::InitDone => {}
        }
    }
    panic!("Secret watcher ended");
}

async fn mcp_template_listener(state: AppState) {
    let mcp_template = Api::<ConfigMap>::all(state.kube_client.clone());
    let label = build_label_query(RESOURCE_TYPE_MCP_TEMPLATE, &[]).unwrap();
    let watch = metadata_watcher(mcp_template, Config::default().labels(&label).timeout(30));
    let mut watch = Box::pin(watch);
    while let Some(event) = watch.try_next().await.unwrap() {
        match event {
            Event::Apply(data) | Event::InitApply(data) | Event::Delete(data) => {
                let namespace = data.namespace().unwrap_or_else(|| "default".to_string());
                let raw_name = data.name_any();
                tokio::spawn(handle_delete_mcp_template(
                    state.kube_store.clone(),
                    namespace,
                    raw_name,
                ));
            }
            Event::Init | Event::InitDone => {}
        }
    }
    panic!("McpTemplate watcher ended");
}

async fn resource_limit_listener(state: AppState) {
    let resource_limit_store = Api::<ConfigMap>::all(state.kube_client.clone());
    let label = build_label_query(RESOURCE_TYPE_RESOURCE_LIMIT, &[]).unwrap();
    let watch = metadata_watcher(
        resource_limit_store,
        Config::default().labels(&label).timeout(30),
    );
    let mut watch = Box::pin(watch);
    while let Some(event) = watch.try_next().await.unwrap() {
        match event {
            Event::Apply(data) | Event::InitApply(data) | Event::Delete(data) => {
                let namespace = data.namespace().unwrap_or_else(|| "default".to_string());
                let raw_name = data.name_any();
                tokio::spawn(handle_delete_resource_limit(
                    state.kube_store.clone(),
                    namespace,
                    raw_name,
                ));
            }
            Event::Init | Event::InitDone => {}
        }
    }
    panic!("ResourceLimit watcher ended");
}

async fn namespace_listener(state: AppState) {
    let namespace = Api::<Namespace>::all(state.kube_client.clone());
    let label = build_label_query(RESOURCE_TYPE_NAMESPACE, &[]).unwrap();
    let watch = metadata_watcher(namespace, Config::default().labels(&label).timeout(30));
    let mut watch = Box::pin(watch);
    while let Some(event) = watch.try_next().await.unwrap() {
        match event {
            Event::Apply(data) | Event::InitApply(data) | Event::Delete(data) => {
                let namespace = data.name_any();
                tokio::spawn(handle_delete_namespace(state.kube_store.clone(), namespace));
            }
            Event::Init | Event::InitDone => {}
        }
    }
    panic!("Namespace watcher ended");
}

async fn handle_delete_namespace(kubestore: KubeStore, namespace: String) {
    let Ok(Some(resource)) = kubestore.namespaces().get(&namespace).await.map_err(|err| {
        tracing::error!("Failed to get namespace {}: {}", namespace, err);
        err
    }) else {
        return;
    };
    if let Some(deletion_timestamp) = &resource.raw.meta().deletion_timestamp {
        let store = kubestore.namespaces();
        let is_deletable = store
            .is_deletable(&namespace)
            .await
            .map_err(|err| {
                tracing::error!(
                    "Failed to check if namespace {} is deletable: {}",
                    namespace,
                    err
                );
                err
            })
            .unwrap_or(false);
        if is_deletable {
            tracing::info!("Deleting namespace: {}", &namespace);
            let result = store
                .delete(&namespace, Some(DeleteOption::remove_finalizer()))
                .await;
            if let Err(err) = result {
                tracing::error!("Error deleting namespace {}: {}", namespace, err);
            } else {
                tracing::info!(
                    "Namespace {} deleted, deletion timestamp: {}",
                    &namespace,
                    &deletion_timestamp.0
                );
            }
        }
    }
}

async fn handle_delete_resource_limit(kubestore: KubeStore, namespace: String, raw_name: String) {
    let Some(name) = decode_k8sname(RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT, &raw_name) else {
        tracing::error!(
            "Failed to decode resource limit name: {}, it must start with {}-",
            raw_name,
            RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT
        );
        return;
    };
    let Ok(Some(resource)) = kubestore.resource_limits().get(&name).await.map_err(|err| {
        tracing::error!(
            "Failed to get resource limit {}/{}: {}",
            &namespace,
            &raw_name,
            err
        );
        err
    }) else {
        tracing::info!(
            "ResourceLimit {}/{} not found when processing deletion, can be removed by other pods",
            &namespace,
            &raw_name
        );
        return;
    };
    if let Some(deletion_timestamp) = &resource.raw.meta().deletion_timestamp {
        tracing::debug!(
            "Processing deletion of ResourceLimit: {}, deletion timestamp: {}",
            &name,
            &deletion_timestamp.0
        );
        let store = kubestore.resource_limits();
        let is_deletable = store
            .is_deletable(&name)
            .await
            .map_err(|err| {
                tracing::error!(
                    "Failed to check if resource limit {} is deletable: {}",
                    name,
                    err
                );
                err
            })
            .unwrap_or(false);
        if is_deletable {
            tracing::info!("Deleting resource limit: {}", &name);
            let result = store
                .delete(&name, Some(DeleteOption::remove_finalizer()))
                .await;
            if let Err(err) = result {
                tracing::error!("Error deleting resource limit {}: {}", name, err);
            } else {
                tracing::info!(
                    "ResourceLimit {} deleted, deletion timestamp: {}",
                    &name,
                    &deletion_timestamp.0
                );
                after_delete(kubestore, resource.raw.clone());
            }
        } else {
            tracing::debug!(
                "ResourceLimit {} is not deletable yet, skipping deletion.",
                &name
            );
        }
    }
}

async fn handle_delete_secret(kubestore: KubeStore, namespace: String, raw_name: String) {
    let Some(name) = decode_k8sname(RESOURCE_TYPE_PREFIX_SECRET, &raw_name) else {
        tracing::error!(
            "Failed to decode secret name: {}, it must start with {}-",
            raw_name,
            RESOURCE_TYPE_PREFIX_SECRET
        );
        return;
    };

    let Ok(Some(resource)) = kubestore
        .secrets(Some(namespace.clone()))
        .get(&name)
        .await
        .map_err(|err| {
            tracing::error!(
                "Failed to get resource limit {}/{}: {}",
                &namespace,
                &raw_name,
                err
            );
            err
        })
    else {
        tracing::info!(
            "Secret {}/{} not found when processing deletion, can be removed by other pods",
            &namespace,
            &raw_name
        );
        return;
    };
    if let Some(deletion_timestamp) = &resource.raw.meta().deletion_timestamp {
        tracing::debug!(
            "Processing deletion of Secret: {}/{}, deletion timestamp: {}",
            &namespace,
            &name,
            &deletion_timestamp.0
        );
        let store = kubestore.secrets(Some(namespace.clone()));
        let is_deletable = store
            .is_deletable(&name)
            .await
            .map_err(|err| {
                tracing::error!("Failed to check if secret {} is deletable: {}", name, err);
                err
            })
            .unwrap_or(false);
        if is_deletable {
            tracing::info!("Deleting secret: {}/{}", &namespace, &name);
            let result = store
                .delete(&name, Some(DeleteOption::remove_finalizer()))
                .await;
            if let Err(err) = result {
                tracing::error!("Error deleting secret {}/{}: {}", &namespace, &name, err);
            } else {
                tracing::info!(
                    "Secret {}/{} deleted, deletion timestamp: {}",
                    &namespace,
                    &name,
                    &deletion_timestamp.0
                );
            }
            after_delete(kubestore, resource.raw.clone());
        } else {
            tracing::debug!(
                "Secret {}/{} is not deletable yet, skipping deletion.",
                &namespace,
                &name
            );
        }
    }
}

async fn handle_delete_mcp_template(kubestore: KubeStore, namespace: String, raw_name: String) {
    let Some(name) = decode_k8sname(RESOURCE_TYPE_PREFIX_MCP_TEMPLATE, &raw_name) else {
        tracing::error!(
            "Failed to decode mcp template name: {}, it must start with {}-",
            raw_name,
            RESOURCE_TYPE_PREFIX_MCP_TEMPLATE
        );
        return;
    };
    let Some(resource) = kubestore
        .mcp_templates(Some(namespace.clone()))
        .get(&name)
        .await
        .map_err(|err| {
            tracing::error!(
                "Failed to get resource limit {}/{}: {}",
                &namespace,
                &raw_name,
                err
            );
            err
        })
        .unwrap()
    else {
        tracing::info!(
            "McpTemplate {}/{} not found when processing deletion, can be removed by other pods",
            &namespace,
            &raw_name
        );
        return;
    };
    if let Some(deletion_timestamp) = &resource.raw.meta().deletion_timestamp {
        tracing::debug!(
            "Processing deletion of McpTemplate: {}/{}, deletion timestamp: {}",
            &namespace,
            &name,
            &deletion_timestamp.0
        );
        let store = kubestore.mcp_templates(Some(namespace.clone()));
        let is_deletable = store
            .is_deletable(&name)
            .await
            .map_err(|err| {
                tracing::error!(
                    "Failed to check if mcp template {} is deletable: {}",
                    name,
                    err
                );
                err
            })
            .unwrap_or(false);
        if is_deletable {
            tracing::info!("Deleting mcp template: {}/{}", &namespace, &name);
            let result = store
                .delete(&name, Some(DeleteOption::remove_finalizer()))
                .await;
            if let Err(err) = result {
                tracing::error!(
                    "Error deleting mcp template {}/{}: {}",
                    &namespace,
                    &name,
                    err
                );
            } else {
                tracing::info!(
                    "McpTemplate {}/{} deleted, deletion timestamp: {}",
                    &namespace,
                    &name,
                    &deletion_timestamp.0
                );
                after_delete(kubestore, resource.raw.clone());
            }
        } else {
            tracing::debug!(
                "McpTemplate {}/{} is not deletable yet, skipping deletion.",
                &namespace,
                &name
            );
        }
    }
}

fn after_delete<T: ResourceExt<DynamicType = ()> + Send + 'static>(
    kubestore: KubeStore,
    resource: T,
) {
    let namespace = resource
        .namespace()
        .unwrap_or_else(|| "default".to_string());
    resource
        .labels()
        .keys()
        .filter_map(filter_relpath)
        .for_each(|(r#type, name)| {
            tracing::debug!("Found dependency: {} {}", r#type, name);
            match r#type.as_str() {
                RESOURCE_TYPE_NAMESPACE => {
                    let kubestore = kubestore.clone();
                    let namespace = name.clone();
                    tokio::spawn(async move {
                        handle_delete_namespace(kubestore, namespace).await;
                    });
                }
                RESOURCE_TYPE_SECRET => {
                    let kubestore = kubestore.clone();
                    let namespace = namespace.clone();
                    let raw_name = encode_k8sname(RESOURCE_TYPE_PREFIX_SECRET, &name);
                    tokio::spawn(async move {
                        handle_delete_secret(kubestore, namespace, raw_name).await;
                    });
                }
                RESOURCE_TYPE_MCP_TEMPLATE => {
                    let kubestore = kubestore.clone();
                    let namespace = namespace.clone();
                    let raw_name = encode_k8sname(RESOURCE_TYPE_PREFIX_MCP_TEMPLATE, &name);
                    tokio::spawn(async move {
                        handle_delete_mcp_template(kubestore, namespace, raw_name).await;
                    });
                }
                RESOURCE_TYPE_RESOURCE_LIMIT => {
                    let kubestore = kubestore.clone();
                    let namespace = kubestore.default_namespace().to_string();
                    let raw_name = encode_k8sname(RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT, &name);
                    tokio::spawn(async move {
                        handle_delete_resource_limit(kubestore, namespace, raw_name).await;
                    });
                }
                _ => {
                    tracing::warn!(
                        "Unexpected dependency type '{}' with name '{}' on Secret {}",
                        r#type,
                        name,
                        resource.name_any()
                    );
                }
            }
        });
}
