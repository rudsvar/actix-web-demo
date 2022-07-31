build:
	cargo build

sqlx-prepare:
	cargo sqlx prepare -- --lib

build-offline:
	SQLX_OFFLINE=true cargo build

run:
	RUST_LOG=info,tracing_actix_web=warn,actix_web_demo=debug,sqlx=off cargo run

run-release:
	RUST_LOG=info,actix_web_demo=info,sqlx=off cargo run --release

test:
	RUST_LOG=info,tracing_actix_web=off,actix_web_demo=debug,sqlx=off cargo test
