use std::sync::Arc;

use kube::Client;
use rmcp::transport::{streamable_http_server::session::local::LocalSessionManager, StreamableHttpService};

use crate::passmcp::PassthroughMcpService;

#[derive(Clone)]
pub struct AppState {
    pub kube_client: Client,
    pub local_session_manager: Arc<LocalSessionManager>,
}
