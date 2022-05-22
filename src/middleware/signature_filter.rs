//! Middleware for appending headers to responses.

use crate::security::signature::{self, SignatureHeader};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::{
    future::{LocalBoxFuture, Ready},
    FutureExt,
};
use std::collections::HashMap;

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct SignatureFilterService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SignatureFilterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers();
        tracing::info!("Getting authorization header");
        let signature_header = headers.get("Authorization").unwrap().to_str().unwrap();
        let signature_header: SignatureHeader = signature_header.parse().unwrap();
        tracing::info!("Got signature {}", signature_header);
        let mut headers2: HashMap<&str, Vec<&str>> = HashMap::new();
        let mandatory_headers = vec![];
        for &h in &mandatory_headers {
            let values: Vec<&str> = headers.get_all(h).map(|h| h.to_str().unwrap()).collect();
            headers2.insert(h, values);
        }
        let request_target = format!("{} {}", req.method(), req.uri());
        headers2.insert("(request-target)", vec![request_target.as_str()]);
        let signature_string = signature::signature_string(&mandatory_headers, &headers2);
        tracing::info!("Verifying signature string {}", signature_string);
        let actual_signature = base64::encode(signature::sign(signature_string.as_bytes()));
        tracing::info!("Actual signature in base64 {}", actual_signature);
        let provided_signature = base64::decode(signature_header.signature()).unwrap();
        let verified = signature::verify(signature_string.as_bytes(), &provided_signature);
        let res = self.service.call(req);
        async move {
            if verified {
                tracing::info!("Signature verified");
                res.await
            } else {
                panic!("Signature validation failed")
            }
        }
        .boxed_local()
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug)]
pub struct SignatureFilter;

impl<S, B> Transform<S, ServiceRequest> for SignatureFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = SignatureFilterService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(SignatureFilterService { service }))
    }
}
