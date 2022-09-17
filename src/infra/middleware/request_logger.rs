//! Middleware adding authentication data to requests.

use crate::{
    infra::security::jwt::Claims,
    repository::request_repository::{self, RequestBuilder},
    DbPool,
};
use actix_http::{
    body::{BoxBody, MessageBody},
    HttpMessage,
};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web::{Bytes, Data},
};
use chrono::Utc;
use futures::future::{LocalBoxFuture, Ready};
use std::rc::Rc;

use super::digest_filter::clone_request_body;

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct RequestLoggerService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>>,
    S: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            let db = req.app_data::<Data<DbPool>>().expect("no db available");
            let mut conn = db.acquire().await.unwrap();

            // Log request information
            let body = clone_request_body(&mut req).await;
            let mut request = RequestBuilder::default();
            let request = request
                .ip(req.connection_info().peer_addr().unwrap().to_string())
                .request_method(req.method().to_string())
                .request_uri(req.uri().to_string())
                .request_body(String::from_utf8(body.to_vec()).unwrap())
                .request_time(Utc::now());

            // Set user id
            if let Some(claims) = req.extensions().get::<Claims>() {
                request.user_id(claims.id());
            }

            // Call inner service
            let start = std::time::Instant::now();
            let resp = svc.call(req).await?;

            let (body, resp) = clone_response_body(resp).await;

            // Log response information
            request
                .response_code(resp.status().as_u16() as i32)
                .response_time_ms(start.elapsed().as_millis() as i32);
            if let Ok(string_body) = String::from_utf8(body.to_vec()) {
                request.response_body(string_body);
            }
            let request = request.build().unwrap();

            // Store request information
            request_repository::store_request(&mut conn, &request)
                .await
                .unwrap();

            Ok(resp)
        })
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug, Default)]
pub struct RequestLogger {}

impl RequestLogger {
    /// Constructs a new [`Authentication`] instance.
    pub fn new() -> Self {
        RequestLogger::default()
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = S::Error;
    type InitError = ();
    type Transform = RequestLoggerService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(RequestLoggerService {
            service: Rc::new(service),
        }))
    }
}

/// Extracts and clones the response body.
pub async fn clone_response_body<B: MessageBody>(
    req: ServiceResponse<B>,
) -> (Bytes, ServiceResponse<BoxBody>) {
    let (parts, res) = req.into_parts();
    let (h, b) = res.into_parts();
    let bytes = actix_http::body::to_bytes(b)
        .await
        .unwrap_or_else(|_| Bytes::new());
    let new_http_res = h.set_body(BoxBody::new(bytes.clone()));
    let new_res = ServiceRequest::from_request(parts).into_response(new_http_res);
    (bytes, new_res)
}
