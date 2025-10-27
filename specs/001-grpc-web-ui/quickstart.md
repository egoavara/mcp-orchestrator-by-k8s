# Quickstart: gRPC Web UI Development

**Feature**: gRPC Service Management Web UI  
**Date**: 2025-10-27  
**Phase**: 1 - Design

## Overview

This guide covers how to set up, develop, and build the Yew-based frontend for the MCP Orchestrator.

---

## Prerequisites

### Required Software

- **Rust**: 1.75+ (matching backend version)
- **wasm32-unknown-unknown target**: `rustup target add wasm32-unknown-unknown`
- **Trunk**: Build tool for Yew applications
  ```bash
  cargo install --locked trunk
  ```
- **Backend**: MCP Orchestrator backend must be running on `localhost:8080`

### Optional Tools

- **wasm-opt**: WASM optimizer (installed automatically by Trunk)
- **Browser Dev Tools**: Chrome/Firefox for debugging

---

## Project Structure

```
crates/mcp-orchestrator-front/
├── src/                  # Source code
│   ├── lib.rs           # Entry point
│   ├── app.rs           # Main app with router
│   ├── api/             # gRPC client layer
│   ├── components/      # Reusable UI components
│   ├── pages/           # Page-level components
│   ├── models/          # Data models (protobuf types)
│   ├── hooks/           # Custom Yew hooks
│   ├── routes.rs        # Route definitions
│   └── utils/           # Utility functions
├── dist/                # Build output (gitignored)
├── index.html           # HTML entry point
├── styles.css           # Global styles
├── Cargo.toml           # Package manifest
└── Trunk.toml           # Trunk configuration
```

---

## Quick Start

### 1. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install --locked trunk
```

### 2. Start Backend

In a separate terminal, start the backend gRPC service:

```bash
cd /workspaces/mcp-orchestrator-by-k8s
cargo run --bin mcp-orchestrator
```

Backend should be running on `http://localhost:8080`.

### 3. Run Frontend Development Server

```bash
cd crates/mcp-orchestrator-front
trunk serve
```

The development server will:
- Compile Rust to WASM
- Start serving on `http://localhost:8000`
- Watch for file changes (hot reload enabled)
- Proxy API requests to backend at `localhost:8080`

### 4. Open Browser

Navigate to:
```
http://localhost:8000
```

You should see the MCP Orchestrator UI.

---

## Development Workflow

### File Structure Guidelines

**Max 300 lines per file** (project guideline). If a file grows beyond this:
1. Extract reusable logic into `utils/`
2. Split complex components into subcomponents
3. Move data models to separate files in `models/`

### Hot Reload

Trunk automatically rebuilds on file changes. Changes appear in the browser within seconds.

**Rebuild Triggers**:
- `.rs` files in `src/`
- `index.html`
- `styles.css`

**No Reload Needed**:
- Backend changes (restart backend separately)
- `Cargo.toml` changes (restart Trunk)

### Debugging

**Browser DevTools**:
```javascript
// Check WASM logs
console.log // Rust log::info! appears here
```

**Rust Logging**:
```rust
use log::{info, warn, error};

info!("User clicked button");
warn!("API call slow: {}ms", elapsed);
error!("Failed to load data: {}", err);
```

**Enable logging in `lib.rs`**:
```rust
#[wasm_bindgen(start)]
pub fn start() {
    wasm_logger::init(wasm_logger::Config::default());
}
```

---

## Building for Production

### Development Build

```bash
trunk build
```

Output: `dist/` directory with unoptimized WASM.

### Production Build

```bash
trunk build --release
```

Features:
- Optimized WASM (wasm-opt)
- Minified JS
- Smaller bundle size
- Output in `dist/`

### Deploy

Copy `dist/` contents to your web server:

```bash
# Example: Deploy to Nginx
cp -r dist/* /var/www/html/

# Example: Deploy to S3
aws s3 sync dist/ s3://your-bucket/ --delete
```

**Backend URL Configuration**:

For production, update API base URL to match your backend deployment:

```rust
// src/api/client.rs
pub fn get_base_url() -> String {
    #[cfg(debug_assertions)]
    {
        "http://localhost:8080".to_string()
    }
    
    #[cfg(not(debug_assertions))]
    {
        web_sys::window()
            .unwrap()
            .location()
            .origin()
            .unwrap() // Use same origin as UI
    }
}
```

---

## Configuration

### Trunk.toml

```toml
[build]
target = "index.html"
release = false
dist = "dist"
public_url = "/"

[watch]
ignore = ["dist"]

[serve]
address = "127.0.0.1"
port = 8000
open = false  # Set to true to auto-open browser

# Proxy API requests to backend
[[proxy]]
backend = "http://localhost:8080"
```

### Cargo.toml

Key dependencies:

```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-router = "0.18"
yewdux = "0.10"  # State management
tonic-web-wasm-client = "0.8"  # gRPC-Web client
gloo-net = "0.4"  # HTTP client utilities
proto = { path = "../proto" }  # Shared protobuf types
```

---

## Common Tasks

### Add a New Page

1. **Create page component**:
   ```bash
   touch src/pages/my_feature/mod.rs
   touch src/pages/my_feature/list.rs
   ```

2. **Define route** in `src/routes.rs`:
   ```rust
   #[derive(Clone, Routable, PartialEq)]
   pub enum Route {
       // ...
       #[at("/my-feature")]
       MyFeature,
   }
   ```

3. **Add route handler** in `src/app.rs`:
   ```rust
   fn switch(routes: Route) -> Html {
       match routes {
           Route::MyFeature => html! { <MyFeaturePage /> },
           // ...
       }
   }
   ```

### Add a New API Call

1. **Create API module** in `src/api/my_resource.rs`:
   ```rust
   use tonic_web_wasm_client::Client;
   use proto::mcp::orchestrator::v1::*;
   
   pub async fn list_my_resources() -> Result<Vec<MyResource>, String> {
       let client = Client::new(get_base_url());
       let mut grpc_client = MyServiceClient::new(client);
       
       let request = tonic::Request::new(ListMyResourcesRequest {});
       let response = grpc_client.list_my_resources(request).await
           .map_err(|e| format!("gRPC error: {}", e))?;
       
       Ok(response.into_inner().data)
   }
   ```

2. **Call from component**:
   ```rust
   use_effect_with((), |_| {
       wasm_bindgen_futures::spawn_local(async move {
           match list_my_resources().await {
               Ok(data) => { /* update state */ },
               Err(e) => { /* handle error */ },
           }
       });
       || ()
   });
   ```

### Add Global State

1. **Define store** in `src/models/state.rs`:
   ```rust
   use yewdux::prelude::*;
   
   #[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
   #[store(storage = "local")]
   pub struct MyState {
       pub my_field: String,
   }
   ```

2. **Use in component**:
   ```rust
   let (state, dispatch) = use_store::<MyState>();
   
   let update_state = dispatch.reduce_mut_callback(|state| {
       state.my_field = "new value".to_string();
   });
   ```

---

## Troubleshooting

### WASM Build Fails

**Error**: `error[E0432]: unresolved import`

**Solution**: Check that `proto` crate has correct features:
```toml
# crates/proto/Cargo.toml
[dependencies]
tonic = { version = "0.14", default-features = false, features = ["codegen", "prost"] }
# NO transport feature!
```

### Hot Reload Not Working

**Solution**:
1. Stop Trunk (`Ctrl+C`)
2. Clean build: `trunk clean`
3. Restart: `trunk serve`

### CORS Errors in Browser

**Error**: `Access to fetch at 'http://localhost:8080' from origin 'http://localhost:8000' has been blocked by CORS policy`

**Solution**: Verify backend has CORS configured:
```rust
// Backend main.rs should have:
.layer(CorsLayer::permissive())
```

### gRPC-Web 404 Not Found

**Error**: `POST http://localhost:8080/mcp.orchestrator.v1.McpOrchestratorService/ListNamespaces 404`

**Solution**: Check that:
1. Backend is running
2. Backend has `tonic_web::GrpcWebLayer` enabled
3. Service path matches proto package name

### State Not Persisting

**Problem**: Yewdux state resets on page refresh

**Solution**: Add storage attribute:
```rust
#[derive(Store)]
#[store(storage = "local")]  // or "session"
struct MyState { ... }
```

---

## Testing

### Unit Tests

```bash
cargo test --target wasm32-unknown-unknown
```

### Integration Tests (Future)

```bash
# Requires wasm-pack
wasm-pack test --headless --firefox
```

---

## Performance

### Bundle Size

Check WASM bundle size:
```bash
trunk build --release
ls -lh dist/*.wasm
```

**Target**: <500 KB for initial load

### Optimization Tips

1. **Code Splitting**: Use `yew::Suspense` for lazy loading
2. **Minimize Dependencies**: Avoid heavy crates
3. **Enable LTO** in `Cargo.toml`:
   ```toml
   [profile.release]
   lto = true
   opt-level = 'z'
   ```

---

## Browser Requirements

- **Chrome/Chromium**: 90+
- **Firefox**: 88+
- **Safari**: 14+
- **Edge**: 90+
- **JavaScript**: Required (WASM needs JS glue code)
- **WebAssembly**: Required

---

## Next Steps

After setup:

1. **Familiarize with codebase**: Read `src/lib.rs`, `src/app.rs`
2. **Explore components**: Check `src/components/` for reusable UI
3. **Test API calls**: Try calling backend from browser console
4. **Implement first feature**: Start with simplest resource (Namespace list)

---

## Resources

- **Yew Documentation**: https://yew.rs/docs/next/
- **Yewdux Guide**: https://github.com/intendednull/yewdux
- **Trunk Guide**: https://trunkrs.dev/
- **tonic-web-wasm-client**: https://github.com/devashishdxt/tonic-web-wasm-client
- **Project Spec**: [spec.md](./spec.md)
- **Data Models**: [data-model.md](./data-model.md)
