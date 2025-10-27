use std::sync::Arc;

use anyhow::Context;
use kube::runtime::events::{Recorder, Reporter};
use proto::mcp::orchestrator::v1::mcp_orchestrator_service_server::McpOrchestratorServiceServer;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod config;
mod error;
mod grpc;
mod http;
mod podmcp;
mod service;
mod state;
mod storage;
mod assets;

use config::AppConfig;
use grpc::GrpcService;
use http::router;
use state::AppState;

use crate::{podmcp::PodMcp, storage::store::KubeStore};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load().context("Failed to load configuration")?;
    let ct = CancellationToken::new();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.server.log_level)),
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

    info!(
        "Ensuring default namespace: {}",
        config.kubernetes.namespace
    );
    store
        .ensure_default_namespace()
        .await
        .context("Failed to ensure default namespace")?;
    info!("Default namespace ready");

    let state = AppState {
        kube_store: KubeStore::new(kube_client.clone(), &config.kubernetes.namespace),
        kube_client: kube_client.clone(),
        kube_recorder: Recorder::new(
            kube_client.clone(),
            Reporter {
                controller: "mcp-orchestrator".to_string(),
                instance: config.kubernetes.pod.as_ref().map(|p| p.name.clone()),
            },
        ),
        podmcp: PodMcp::new(KubeStore::new(
            kube_client.clone(),
            &config.kubernetes.namespace,
        )),
        config: Arc::new(config.clone()),
    };

    let grpc_service = GrpcService::new(state.clone());
    let grpc_server = McpOrchestratorServiceServer::new(grpc_service);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .context("Failed to build reflection service")?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let http_router = router().with_state(state.clone());

    let grpc_with_reflection = axum::Router::new()
        .route_service("/mcp.orchestrator.v1.McpOrchestratorService/{*path}", 
            ServiceBuilder::new()
                .layer(tonic_web::GrpcWebLayer::new())
                .service(grpc_server))
        .route_service("/grpc.reflection.v1.ServerReflection/{*path}",
            ServiceBuilder::new()
                .layer(tonic_web::GrpcWebLayer::new())
                .service(reflection_service));

    let app = http_router
        .merge(grpc_with_reflection)
        .fallback(http::fallback::handler);

    let addr = format!("{}:{}", config.server.host, config.server.port);

    info!("Server listening on {} (HTTP + gRPC-Web)", addr);

    tokio::spawn(async {
        service::listeners(state, ct).await;
    });

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.layer(cors)).await?;

    Ok(())
}
