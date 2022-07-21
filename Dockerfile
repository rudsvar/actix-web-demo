FROM rust:1.62.0 AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin actix-web-demo

# We do not need the Rust toolchain to run the binary!
FROM chef AS runtime
WORKDIR /app
RUN apt-get update
RUN apt-get upgrade
RUN apt-get install -y libssl-dev
COPY . .
COPY --from=builder /app/target/release/actix-web-demo /usr/local/bin/actix-web-demo
ENV RUST_LOG info
ENTRYPOINT ["/usr/local/bin/actix-web-demo"]
