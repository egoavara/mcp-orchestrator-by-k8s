use std::sync::Arc;

use anyhow::Context;
use k8s_openapi::api::core::v1::Namespace;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use rmcp::transport::{
    streamable_http_server::session::local::LocalSessionManager, StreamableHttpService,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::{passmcp::SampleSessionManager, state::AppState};

pub mod assets;
pub mod kube;
pub mod passmcp;
pub mod route;
pub mod service;
pub mod state;

async fn ensure_namespace(client: ::kube::Client, namespace: &str) -> anyhow::Result<()> {
    use ::kube::api::PostParams;
    use ::kube::Api;

    let namespaces: Api<Namespace> = Api::all(client);

    match namespaces.get(namespace).await {
        Ok(_) => {
            info!("Namespace '{}' already exists", namespace);
            Ok(())
        }
        Err(::kube::Error::Api(err)) if err.code == 404 => {
            info!("Namespace '{}' not found, creating...", namespace);

            let ns = Namespace {
                metadata: ObjectMeta {
                    name: Some(namespace.to_string()),
                    ..Default::default()
                },
                ..Default::default()
            };

            match namespaces.create(&PostParams::default(), &ns).await {
                Ok(_) => {
                    info!("Namespace '{}' created successfully", namespace);
                    Ok(())
                }
                Err(::kube::Error::Api(err)) if err.code == 409 => {
                    info!(
                        "Namespace '{}' already exists (created by another process)",
                        namespace
                    );
                    Ok(())
                }
                Err(e) => Err(e).context("Failed to create namespace"),
            }
        }
        Err(e) => {
            warn!("Failed to check namespace: {}", e);
            Err(e.into())
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting MCP Orchestrator");

    let kube_client = ::kube::Client::try_default()
        .await
        .context("Failed to initialize Kubernetes client")?;

    info!("Kube client initialized successfully");

    let namespace = std::env::var("KUBE_NAMESPACE").unwrap_or_else(|_| "mcp-servers".to_string());
    ensure_namespace(kube_client.clone(), &namespace).await?;

    let local_session_manager = Arc::new(SampleSessionManager::default());
    let state = AppState {
        kube_client,
        local_session_manager: local_session_manager.clone(),
    };

    let app = route::router()
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);

    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
