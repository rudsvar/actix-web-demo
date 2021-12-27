use crate::routes::{client_context, health_check, post_subscription};
use actix_http::header::{HeaderName, HeaderValue};
use actix_web::dev::Service;
use actix_web::{dev::Server, web, App, HttpServer};
use futures::future::FutureExt;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, db_pool: PgPool) -> std::io::Result<Server> {
    let pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap_fn(|mut req, srv| {
                req.headers_mut().append(
                    HeaderName::from_static("foo"),
                    HeaderValue::from_static("foo!"),
                );
                tracing::info!("Modifying request: {:?}", req);
                srv.call(req).map(|res| {
                    res.map(|mut res| {
                        res.headers_mut().append(
                            HeaderName::from_static("bar"),
                            HeaderValue::from_static("bar"),
                        );
                        tracing::info!("Modifying response: {:?}", res);
                        res
                    })
                })
            })
            .app_data(pool.clone())
            .service(health_check)
            .service(post_subscription)
            .route("/client_context", web::get().to(client_context))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
