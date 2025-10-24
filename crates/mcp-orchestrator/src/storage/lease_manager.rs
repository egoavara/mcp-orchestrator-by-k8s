use chrono::Utc;
use k8s_openapi::api::coordination::v1::{Lease, LeaseSpec};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::MicroTime;
use kube::{
    api::{DeleteParams, ObjectMeta, Patch, PatchParams, PostParams},
    Api, Client,
};
use serde_json::json;
use std::time::Duration;

use crate::error::AppError;

const DEFAULT_LEASE_DURATION_SECS: i32 = 15;

pub struct LeaseManager {
    api: Api<Lease>,
    lease_name: String,
    holder_identity: String,
    lease_duration_secs: i32,
}

impl LeaseManager {
    pub fn new(
        client: Client,
        namespace: &str,
        lease_name: &str,
        holder_identity: &str,
    ) -> Self {
        Self {
            api: Api::namespaced(client, namespace),
            lease_name: lease_name.to_string(),
            holder_identity: holder_identity.to_string(),
            lease_duration_secs: DEFAULT_LEASE_DURATION_SECS,
        }
    }

    pub fn with_duration(mut self, duration_secs: i32) -> Self {
        self.lease_duration_secs = duration_secs;
        self
    }

    pub async fn try_acquire(&self) -> Result<bool, AppError> {
        let now = MicroTime(Utc::now());

        let lease = Lease {
            metadata: ObjectMeta {
                name: Some(self.lease_name.clone()),
                ..Default::default()
            },
            spec: Some(LeaseSpec {
                holder_identity: Some(self.holder_identity.clone()),
                lease_duration_seconds: Some(self.lease_duration_secs),
                acquire_time: Some(now.clone()),
                renew_time: Some(now),
                lease_transitions: Some(0),
                preferred_holder: None,
                strategy: None,
            }),
        };

        match self.api.create(&PostParams::default(), &lease).await {
            Ok(_) => Ok(true),
            Err(kube::Error::Api(e)) if e.code == 409 => self.try_renew().await,
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub async fn try_renew(&self) -> Result<bool, AppError> {
        let existing = match self.api.get(&self.lease_name).await {
            Ok(lease) => lease,
            Err(kube::Error::Api(e)) if e.code == 404 => {
                return Box::pin(self.try_acquire()).await;
            }
            Err(e) => return Err(AppError::from(e)),
        };

        let current_holder = existing
            .spec
            .as_ref()
            .and_then(|s| s.holder_identity.as_ref());

        if current_holder == Some(&self.holder_identity) {
            let now = MicroTime(Utc::now());
            let patch = json!({
                "spec": {
                    "renewTime": now,
                }
            });

            self.api
                .patch(
                    &self.lease_name,
                    &PatchParams::default(),
                    &Patch::Merge(&patch),
                )
                .await
                .map_err(AppError::from)?;

            Ok(true)
        } else {
            let lease_expired = existing
                .spec
                .as_ref()
                .and_then(|s| {
                    let renew_time = s.renew_time.as_ref()?.0;
                    let duration = s.lease_duration_seconds?;
                    Some(
                        Utc::now().signed_duration_since(renew_time).num_seconds()
                            > duration as i64,
                    )
                })
                .unwrap_or(true);

            if lease_expired {
                self.api
                    .delete(&self.lease_name, &DeleteParams::default())
                    .await
                    .map_err(AppError::from)?;
                
                tokio::time::sleep(Duration::from_millis(100)).await;
                Box::pin(self.try_acquire()).await
            } else {
                Ok(false)
            }
        }
    }

    pub async fn release(&self) -> Result<(), AppError> {
        match self.api.get(&self.lease_name).await {
            Ok(existing) => {
                let current_holder = existing
                    .spec
                    .as_ref()
                    .and_then(|s| s.holder_identity.as_ref());

                if current_holder == Some(&self.holder_identity) {
                    self.api
                        .delete(&self.lease_name, &DeleteParams::default())
                        .await
                        .map_err(AppError::from)?;
                }
                Ok(())
            }
            Err(kube::Error::Api(e)) if e.code == 404 => Ok(()),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn holder_identity(&self) -> &str {
        &self.holder_identity
    }
}

pub fn resource_lock_name(resource_type: &str, namespace: &str, name: &str) -> String {
    format!("lock-{}-{}-{}", resource_type, namespace, name)
}
