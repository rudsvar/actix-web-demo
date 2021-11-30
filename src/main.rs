#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    actix_web_demo::run()?.await
}
