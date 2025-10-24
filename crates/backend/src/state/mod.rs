use std::sync::Arc;

use kube::Client;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

#[derive(Clone)]
pub struct AppState {
    pub kube_client: Client,
    pub local_session_manager: Arc<LocalSessionManager>,
}
