use anyhow::Context;
use proto::mcp::orchestrator::v1::mcp_orchestrator_service_server::McpOrchestratorServiceServer;
use tower_http::cors::{Any, CorsLayer};
use tower::ServiceBuilder;
use tracing::info;

mod config;
mod error;
mod grpc;
mod http;
mod service;
mod state;
mod storage;

use config::AppConfig;
use grpc::GrpcService;
use http::create_http_router;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()
        .context("Failed to load configuration")?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new(&config.server.log_level)
                })
        )
        .init();

    info!("Starting MCP Orchestrator (gRPC-Web)");
    info!("Configuration loaded:");
    info!("  Server: {}:{}", config.server.host, config.server.port);
    info!("  Log level: {}", config.server.log_level);
    info!("  Kubernetes namespace: {}", config.kubernetes.namespace);
    if let Some(ctx) = &config.kubernetes.context {
        info!("  Kubernetes context: {}", ctx);
    }

    let kube_client = kube::Client::try_default()
        .await
        .context("Failed to initialize Kubernetes client")?;

    info!("Kubernetes client initialized");

    let store = storage::store::KubeStore::new(kube_client.clone(), &config.kubernetes.namespace);
    
    info!("Ensuring default namespace: {}", config.kubernetes.namespace);
    store.ensure_default_namespace()
        .await
        .context("Failed to ensure default namespace")?;
    info!("Default namespace ready");

    let state = AppState { 
        kube_client,
        default_namespace: config.kubernetes.namespace.clone(),
    };

    let grpc_service = GrpcService::new(state.clone());
    let grpc_server = McpOrchestratorServiceServer::new(grpc_service);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let grpc_service_with_web = ServiceBuilder::new()
        .layer(tonic_web::GrpcWebLayer::new())
        .service(grpc_server);

    let http_router = create_http_router().with_state(state);

    let app = http_router.fallback_service(grpc_service_with_web);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    
    info!("Server listening on {} (HTTP + gRPC-Web)", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.layer(cors)).await?;

    Ok(())
}
