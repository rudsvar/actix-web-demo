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

- You can generate a new elliptic curve key pair with the following commands.
    ```sh
    # Generate private key
    openssl ecparam -name prime256v1 -genkey -noout -out private.ec.key
    # Convert to pkcs8 format and encrypt
    openssl pkcs8 -topk8 -in private.ec.key -out private.pem
    # Generate a corresponding public key
    openssl ec -in private.pem -pubout -out public.pem
    ```
    See <https://stackoverflow.com/questions/15686821/generate-ec-keypair-from-openssl-command-line> for more information.

# Benchmarks

To discover performance bottlenecks, take a look at https://github.com/flamegraph-rs/flamegraph. Note that you might have issues installing it in WSL; if so, take a look at https://stackoverflow.com/a/65276025.

```
PERF=/usr/lib/linux-tools/5.4.0-120-generic/perf cargo flamegraph
```
