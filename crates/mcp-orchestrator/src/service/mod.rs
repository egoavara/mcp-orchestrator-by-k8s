use futures::{TryFutureExt, TryStreamExt};
use k8s_openapi::api::core::v1::{ConfigMap, Namespace, Secret};
use kube::{
    Api, Resource, ResourceExt,
    runtime::{
        WatchStreamExt,
        metadata_watcher,
        watcher::Config,
    },
};

use crate::{
    state::AppState,
    storage::{
        labels::LABEL_MANAGED_BY_QUERY,
        util_delete::DeleteOption,
    },
};

pub async fn listeners(state: AppState) {
    // let pod = Api::<Pod>::all(state.kube_client.clone());
    // let secret = Api::<Secret>::all(state.kube_client.clone());

    // metadata_watcher(pod, )
    futures::join!(
        // metadata_watcher(pod, state.clone()),
        // secret_watcher(state.clone()),
        secret_listener(state.clone()),
        mcp_template_listener(state.clone()),
        resource_limit_listener(state.clone()),
        namespace_listener(state.clone()),
    );
}

async fn secret_listener(state: AppState) {
    let secret = Api::<Secret>::all(state.kube_client.clone());
}

async fn mcp_template_listener(state: AppState) {
    let mcp_template = Api::<ConfigMap>::all(state.kube_client.clone());
}

async fn resource_limit_listener(state: AppState) {
    let resource_limit_store = Api::<ConfigMap>::all(state.kube_client.clone());
}

async fn namespace_listener(state: AppState) {
    let namespace = Api::<Namespace>::all(state.kube_client.clone());
    let watch = metadata_watcher(
        namespace,
        Config::default().labels(LABEL_MANAGED_BY_QUERY).timeout(10),
    )
    .applied_objects();
    let mut watch = Box::pin(watch);
    while let Some(ns) = watch.try_next().await.unwrap() {
        tracing::info!("Namespace changed: {}", ns.name_any());
        handle_namespace(&state, ns).await;
    }
    panic!("Namespace watcher ended");
}

async fn handle_namespace<T: ResourceExt<DynamicType = ()>>(state: &AppState, ns: T) {
    let target_ns = ns.name_any();
    if let Some(deletion_timestamp) = &ns.meta().deletion_timestamp {
        let namespace = state.kube_store.namespaces();
        let is_deletable = namespace
            .is_deletable(&target_ns)
            .await
            .map_err(|err| {
                tracing::error!(
                    "Failed to check if namespace {} is deletable: {}",
                    target_ns,
                    err
                );
                err
            })
            .unwrap_or(false);
        if is_deletable {
            tracing::info!("Deleting namespace: {}", &target_ns);
            let result = namespace
                .delete(&target_ns, Some(DeleteOption::force()))
                .await;
            if let Err(err) = result {
                tracing::error!("Error deleting namespace {}: {}", target_ns, err);
            } else {
                tracing::info!(
                    "Namespace {} deleted, deletion timestamp: {}",
                    &target_ns,
                    &deletion_timestamp.0
                );
            }
        }
    }
}
