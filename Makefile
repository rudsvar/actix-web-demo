build:
	cargo build

run:
	RUST_LOG=info,tracing_actix_web=off,actix_web_demo=debug,sqlx=off cargo run

test:
	RUST_LOG=info,tracing_actix_web=off,actix_web_demo=debug,sqlx=off cargo test
