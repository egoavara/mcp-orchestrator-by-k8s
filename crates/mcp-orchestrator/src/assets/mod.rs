use include_dir::Dir;

pub static STATIC_ASSETS: Dir<'_> =
    include_dir::include_dir!("$CARGO_WORKSPACE_DIR/crates/mcp-orchestrator-front/dist");
