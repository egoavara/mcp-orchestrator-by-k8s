use kube::{runtime::events::Recorder, Client};

use crate::storage::store::KubeStore;

#[derive(Clone)]
pub struct AppState {
    pub kube_client: Client,
    pub kube_store: KubeStore,
    pub kube_recorder: Recorder,
}
