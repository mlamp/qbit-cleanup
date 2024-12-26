# --------------------------------------------------------
# 1) Builder Stage: Build the Rust binary
# --------------------------------------------------------
    FROM rust:latest AS builder

    # Create a new empty shell project
    WORKDIR /usr/src/qbit-cleanup
    
    # Copy your Cargo.toml and Cargo.lock first (for dependency caching)
    COPY Cargo.toml Cargo.lock ./
    
    # Copy the source code
    COPY src ./src
    
    # If you have additional files (e.g., README.md, .env), copy them too:
    # COPY README.md ./
    
    # Build in release mode
    RUN cargo build --release
    
    # --------------------------------------------------------
    # 2) Final Stage: Minimal runtime container
    # --------------------------------------------------------
    FROM debian:stable-slim
    
    # Install any OS dependencies your CLI may need at runtime (optional)
    RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*
    
    # Copy the final binary from builder image
    COPY --from=builder /usr/src/qbit-cleanup/target/release/qbit-cleanup /usr/local/bin/qbit-cleanup
    
    # Declare what the container should run by default
    ENTRYPOINT ["qbit-cleanup"]
    