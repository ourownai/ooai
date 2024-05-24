# Build stage
FROM rust:bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    software-properties-common \
    libssl-dev \
    cmake \
    python3-dev \
    curl \
    pkg-config

# Set the working directory
WORKDIR /app

# Copy the manifest files to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Download dependencies
RUN mkdir -p src/bin && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/bin/bigbot.rs && \
    cargo build --release

# Copy the source code
COPY ./src ./src

# Build the project with release profile
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim as runtime

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    python3-dev && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the binary from the build stage
COPY --from=builder /app/target/release/bigbot /usr/local/bin/bigbot

# Expose the necessary port (replace 8080 with the actual port your application uses)
EXPOSE 8080

# Set the entrypoint to the binary
ENTRYPOINT ["bigbot"]
