FROM --platform=$BUILDPLATFORM rust:1.90-bookworm AS base
WORKDIR /app

# Install protobuf compiler and proto definitions
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    protobuf-compiler libprotobuf-dev \
    gcc-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf \
    gcc-x86-64-linux-gnu \
    libc6-dev-arm64-cross \
    libc6-dev-armhf-cross \
    libc6-dev-amd64-cross \
    && curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-chef
RUN cargo binstall --no-confirm --no-symlinks --locked cargo-chef trunk

# ============================================================================
# Stage 2: Planner - Dependency planning
# Purpose: Prepares a list of dependencies to be cached
# ============================================================================
FROM --platform=$BUILDPLATFORM base AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json --bin mcp-orchestrator

# ============================================================================
# Stage 3: Builder - Cross-compilation setup and buildq
# Purpose: Sets up cross-compilation environment and builds the Rust binary
#          for the target platform (amd64, arm64, or armv7)
# ============================================================================
FROM --platform=$BUILDPLATFORM base AS builder

# Build arguments
ARG TARGETPLATFORM

WORKDIR /app

# Platform to Rust target triple mapping
# Maps Docker platform names to Rust target triples
RUN case "$TARGETPLATFORM" in \
    "linux/amd64") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
    "linux/arm64") echo aarch64-unknown-linux-gnu > /rust_target.txt ;; \
    "linux/arm/v7") echo armv7-unknown-linux-gnueabihf > /rust_target.txt ;; \
    *) echo "ERROR: Unsupported platform $TARGETPLATFORM" >&2 && exit 1 ;; \
    esac

# Add Rust target for cross-compilation
RUN export RUST_TARGET=$(cat /rust_target.txt) && \
    echo "Adding Rust target: $RUST_TARGET" && \
    rustup target add $RUST_TARGET && \
    rustup target add wasm32-unknown-unknown

# Copy dependency recipe from planner stage
COPY --from=planner /app/recipe.json recipe.json

# Cook dependencies (cached layer)
# This layer is cached unless Cargo.toml changes, saving 5-10 minutes on rebuilds
# Only build mcp-orchestrator binary dependencies (exclude wasm frontend)
RUN echo "cargo chef cook for the target platform" && \
    export RUST_TARGET=$(cat /rust_target.txt) && \
    case "$RUST_TARGET" in \
    "x86_64-unknown-linux-gnu") \
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc ;; \
    "aarch64-unknown-linux-gnu") \
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ;; \
    "armv7-unknown-linux-gnueabihf") \
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ;; \
    esac && \
    cargo chef cook --release --target $RUST_TARGET --bin mcp-orchestrator --recipe-path recipe.json

# Copy source code for build
COPY . .

# Build application binary
RUN echo "Build both the wasm frontend and the main orchestrator binary" && \
    export RUST_TARGET=$(cat /rust_target.txt) && \
    case "$RUST_TARGET" in \
    "x86_64-unknown-linux-gnu") \
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc ;; \
    "aarch64-unknown-linux-gnu") \
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc ;; \
    "armv7-unknown-linux-gnueabihf") \
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc ;; \
    esac && \
    cargo build --release --target $RUST_TARGET --bin mcp-orchestrator && \
    mkdir -p /app/build && \
    ls -la target/$RUST_TARGET/release/ && \
    cp target/$RUST_TARGET/release/mcp-orchestrator /app/build/mcp-orchestrator && \
    ls -la /app/build/

# ============================================================================
# Stage 4: Runtime - Minimal runtime image
# Purpose: Minimal Debian-based image with only the binary and runtime deps
#          Final image size: ~80-100MB (vs 2GB+ build image)
# ============================================================================
FROM debian:bookworm-slim AS runtime


# Install runtime dependencies
# - ca-certificates: TLS/HTTPS support (required for Redis TLS, external APIs)
# - tini: Proper PID 1 init system for signal handling and zombie reaping
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tini \
    && rm -rf /var/lib/apt/lists/*

# Copy the correct binary from builder stage
# The builder stage already moved the cross-compiled binary to /app/build/mcp-orchestrator
# Docker buildx ensures the correct builder variant is used for each target platform
COPY --from=builder --chmod=755 /app/build/mcp-orchestrator /usr/local/bin/mcp-orchestrator

# OCI image labels for metadata
# See: https://github.com/opencontainers/image-spec/blob/main/annotations.md
LABEL org.opencontainers.image.title="mcp-orchestrator" \
    org.opencontainers.image.description="Kubernetes-based mcp orchestrator for multiplayer coding playgrounds" \
    org.opencontainers.image.authors="egoavara" \
    org.opencontainers.image.source="https://github.com/egoavara/mcp-orchestrator-by-k8s" \
    org.opencontainers.image.licenses="MIT" \
    org.opencontainers.image.version="0.1.0"

# Use tini as PID 1 for proper signal handling (SIGTERM, SIGINT)
# Tini ensures signals are properly forwarded to the application
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/mcp-orchestrator"]
