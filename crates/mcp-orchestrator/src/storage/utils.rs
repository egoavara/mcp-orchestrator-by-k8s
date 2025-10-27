use std::{collections::BTreeMap, fmt::Debug};

use chrono::Duration;
use kube::{ResourceExt, api::PatchParams};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use tokio::time::sleep;

use crate::error::AppError;

pub async fn add_safe_finalizer<K: Clone + DeserializeOwned + Debug + ResourceExt>(
    api: kube::api::Api<K>,
    name: &str,
    finalizer: &str,
    max_retries: usize,
) -> Result<(), AppError> {
    safe_finalizer_inner(
        api,
        name,
        |mut current_finalizers: Vec<String>| {
            if current_finalizers.iter().any(|f| f == finalizer) {
                None
            } else {
                current_finalizers.push(finalizer.to_string());
                Some(current_finalizers)
            }
        },
        max_retries,
    )
    .await
}
pub async fn del_safe_finalizer<K: Clone + DeserializeOwned + Debug + ResourceExt>(
    api: kube::api::Api<K>,
    name: &str,
    finalizer: &str,
    max_retries: usize,
) -> Result<(), AppError> {
    safe_finalizer_inner(
        api,
        name,
        |mut current_finalizers: Vec<String>| {
            if current_finalizers.iter().any(|f| f == finalizer) {
                current_finalizers.retain(|f| f != finalizer);
                Some(current_finalizers)
            } else {
                None
            }
        },
        max_retries,
    )
    .await
}

async fn safe_finalizer_inner<
    K: Clone + DeserializeOwned + Debug + ResourceExt,
    F: Fn(Vec<String>) -> Option<Vec<String>>,
>(
    api: kube::api::Api<K>,
    name: &str,
    f: F,
    max_retries: usize,
) -> Result<(), AppError> {
    for _ in 0..(max_retries) {
        let resource = api.get(name).await.map_err(AppError::from)?;
        let rversion = resource.resource_version();
        let meta = resource.meta();
        let current_finalizers = meta.finalizers.as_ref().cloned().unwrap_or_else(Vec::new);
        let Some(expected_finalizers) = f(current_finalizers) else {
            break;
        };

        let patch = serde_json::from_value::<json_patch::Patch>(json!([
            {
                "op": "test",
                "path": "/metadata/resourceVersion",
                "value": rversion
            },
            {
                "op": "add",
                "path": "/metadata/finalizers",
                "value": expected_finalizers
            }
        ]))?;
        match api
            .patch(
                name,
                &PatchParams::default(),
                &kube::api::Patch::Json::<()>(patch),
            )
            .await
            .map(|_| false)
            .or_else(|x| {
                if let kube::Error::Api(ref resp) = x
                    && resp.code == 409
                {
                    Ok(true)
                } else {
                    tracing::error!("Failed to modify resource {} cause {}", name, x);
                    Err(AppError::from(x))
                }
            }) {
            Ok(false) => return Ok(()),
            Ok(true) => {
                sleep(std::time::Duration::from_millis(300)).await;
                continue;
            }
            Err(e) => {
                tracing::error!("Failed to modify resource {}: {}, retrying...", name, e);
                sleep(std::time::Duration::from_millis(300)).await;
            }
        }
    }
    Err(AppError::Internal(format!(
        "Failed to modify resource {} after {} retries",
        name, max_retries
    )))
}

pub async fn interval_timeout<
    A,
    R: std::future::Future<Output = Option<A>> + Send,
    F: Fn() -> R,
>(
    duration: Duration,
    max_duration: Duration,
    f: F,
) -> Option<A> {
    let timeout = tokio::time::timeout(max_duration.to_std().unwrap(), async move {
        let mut interval = tokio::time::interval(duration.to_std().unwrap());
        loop {
            interval.tick().await;
            let result = f().await;
            if let Some(result) = result {
                return result;
            }
        }
    })
    .await;
    timeout.ok()
}

pub fn data_elem<S: Serialize>(key: &str, data: &S) -> Result<(String, String), AppError> {
    serde_json::to_string(data)
        .map(|s| (key.to_string(), s))
        .map_err(AppError::SerializationError)
}

pub fn parse_data_elem<D: DeserializeOwned>(
    data: &Option<BTreeMap<String, String>>,
    key: &str,
) -> Result<D, AppError> {
    let Some(map) = data else {
        return Err(AppError::Internal(format!(
            "Data map is None, cannot find key {}",
            key
        )));
    };
    let Some(value) = map.get(key) else {
        return Err(AppError::Internal(format!(
            "Key {} not found in data map",
            key
        )));
    };
    serde_json::from_str::<D>(value).map_err(AppError::SerializationError)
}
