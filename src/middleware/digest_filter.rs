//! Middleware for appending headers to responses.

use actix_http::{body::BoxBody, h1::Payload, HttpMessage};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web::BytesMut,
    HttpResponse,
};
use futures::{
    future::{LocalBoxFuture, Ready},
    StreamExt,
};
use openssl::hash::MessageDigest;
use std::rc::Rc;

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct DigestFilterService<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for DigestFilterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>>,
    S: 'static,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            // Get request body
            let mut body = BytesMut::new();
            let mut stream = req.take_payload();

            while let Some(chunk) = stream.next().await {
                body.extend_from_slice(&chunk.unwrap());
            }

            if !body.is_empty() {
                tracing::info!("Validating digest");

                // Get digest header
                let headers = req.headers();
                let digest_header = headers.get("digest").unwrap();
                let digest_header = digest_header
                    .to_str()
                    .unwrap()
                    .split_once('=')
                    .unwrap()
                    .1
                    .to_string();
                tracing::info!("Got digest header {:?}", digest_header);

                // Compute digest and compare to header
                let digest_body: &[u8] =
                    &openssl::hash::hash(MessageDigest::sha256(), &body).unwrap();
                let digest_body = openssl::base64::encode_block(digest_body);
                if digest_header != digest_body {
                    tracing::info!("Expected digest {:?}", digest_body);
                    return Ok(req
                        .into_response(HttpResponse::Unauthorized())
                        .map_into_boxed_body());
                }
            } else {
                tracing::info!("Empty body, not validating digest");
            }

            // Reset payload
            let (_, mut payload) = Payload::create(true);
            payload.unread_data(body.into());
            req.set_payload(payload.into());

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug)]
pub struct DigestFilter;

impl<S> Transform<S, ServiceRequest> for DigestFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = actix_web::Error>,
    S::Future: 'static,
    S: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = DigestFilterService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(DigestFilterService {
            service: Rc::new(service),
        }))
    }
}
