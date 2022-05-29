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

/// Extracts the request body.
async fn get_request_body(req: &mut ServiceRequest) -> BytesMut {
    let mut body = BytesMut::new();
    let mut stream = req.take_payload();

    while let Some(chunk) = stream.next().await {
        body.extend_from_slice(&chunk.unwrap());
    }

    body
}

/// An error during digest validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigestError {
    /// The digest header is missing.
    MissingHeader,
    /// The header is not a valid digest header.
    InvalidHeader,
    /// The digest header does not match the digest of the body.
    DigestMismatch,
}

/// Extracts the digest header.
async fn get_digest_header(req: &ServiceRequest) -> Result<&str, DigestError> {
    let headers = req.headers();
    let digest_header = headers.get("digest").ok_or(DigestError::MissingHeader)?;
    let digest_header = digest_header
        .to_str()
        .map_err(|_| DigestError::InvalidHeader)?
        .split_once('=')
        .ok_or(DigestError::InvalidHeader)?
        .1;
    Ok(digest_header)
}

/// Validates
async fn validate_digest(req: &ServiceRequest, body: &[u8]) -> Result<(), DigestError> {
    tracing::info!("Validating digest");

    // Get digest header
    let digest_header = get_digest_header(req).await?;

    tracing::info!("Got digest header {:?}", digest_header);

    // Compute digest and compare to header
    let digest_body = openssl::hash::hash(MessageDigest::sha256(), body)
        .map_err(|_| DigestError::InvalidHeader)?;
    let digest_body = openssl::base64::encode_block(&digest_body);
    if digest_header != digest_body {
        tracing::info!("Expected digest {:?}", digest_body);
        return Err(DigestError::DigestMismatch);
    }

    Ok(())
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
            let body = get_request_body(&mut req).await;

            if !body.is_empty() {
                // Validate digest
                let result = validate_digest(&req, &body).await;
                if result.is_err() {
                    return Ok(req
                        .into_response(HttpResponse::Unauthorized())
                        .map_into_boxed_body());
                }
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
