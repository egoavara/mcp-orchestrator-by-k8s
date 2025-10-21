use axum::response::Html;

pub async fn handler() -> Html<&'static str> {
    Html("<h1>MCP Orchestrator</h1><p>API available at /api/mcp</p>")
}
