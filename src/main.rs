use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    actix_web_demo::run(listener)?.await
}
