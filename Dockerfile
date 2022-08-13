FROM lukemathwalker/cargo-chef:latest-rust-1.63-buster AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update
RUN apt-get install -y protobuf-compiler
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin actix-web-demo

FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt-get update
RUN apt-get install -y libssl-dev
COPY --from=builder /app/target/release/actix-web-demo /usr/local/bin
COPY configuration.yaml configuration.yaml
COPY resources resources
ENV RUST_LOG info,tracing_actix_web=warn,actix_web_demo=debug,sqlx=off
ENTRYPOINT ["/usr/local/bin/actix-web-demo"]
