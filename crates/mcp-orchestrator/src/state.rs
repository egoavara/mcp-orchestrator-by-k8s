use std::sync::Arc;

use kube::{Client, runtime::events::Recorder};

use crate::{config::AppConfig, podmcp::PodMcp, storage::store::KubeStore};

#[derive(Clone)]
pub struct AppState {
    pub kube_client: Client,
    pub kube_store: KubeStore,
    pub kube_recorder: Recorder,
    pub podmcp: PodMcp,
    pub config: Arc<AppConfig>,
}
