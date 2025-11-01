use std::sync::Arc;

use kube::{Client, runtime::events::Recorder};
use oidc_auth::AuthManager;

use crate::{config::AppConfig, podmcp::PodMcp, storage::store::KubeStore};

#[derive(Clone)]
pub struct AppState {
    pub kube_client: Client,
    pub kube_store: KubeStore,
    pub _kube_recorder: Recorder,
    pub podmcp: PodMcp,
    pub config: Arc<AppConfig>,
    pub oidc_manager: Option<AuthManager>,
}
