REGISTRY=registry.digitalocean.com/rudsvar
APP_NAME=actix-web-demo

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

docker-build:
	docker build . -t ${REGISTRY}/${APP_NAME}:latest

docker-push:
	docker push ${REGISTRY}/${APP_NAME}:latest

docker-deploy:
	doctl apps create-deployment $(shell doctl apps list --no-header | grep actix-web-demo | cut -f 1 -d ' ') --wait true
