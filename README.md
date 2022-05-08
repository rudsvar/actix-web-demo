# Actix Web Demo

A demo web application made with `actix-web`.

# Software

- Rust <https://rustup.rs/>
- PostgreSQL: <https://www.postgresql.org/>
- SQLx <https://github.com/launchbadge/sqlx>
- Docker: <https://www.docker.com/>
- docker-compose: <https://docs.docker.com/compose/install/>

# Development

1. Start a PostgreSQL database. The easiest way is to run `docker-compose up -d postgres`.
2. We use SQLx to manage our migrations. Set up the database with `sqlx database setup`.
3. Run the application with `cargo run`.

To customize the log level, use the `RUST_LOG` environment variable.

# Useful commands

- Generate documentation: `cargo doc`, open it in a browser with `--open`.
- Learn more about SQLx and migrations: `sqlx -h`

# TODO

- Add authorization with actix-web-grants https://github.com/DDtKey/actix-web-grants
  Find some operation that only administrators should have.
  Make sure users can only send from their own accounts.
- Clean up logging, set better defaults. Maybe move away from the json format (or even better, make it configurable).
- Generate OpenAPI contract and add Swagger UI.
