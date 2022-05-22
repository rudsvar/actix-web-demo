//! Middleware for appending headers to responses.

use crate::security::signature::{self, SignatureHeader};
use actix_http::body::BoxBody;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    HttpResponse,
};
use futures::future::{LocalBoxFuture, Ready};
use std::{collections::HashMap, rc::Rc};

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct SignatureFilterService<S> {
    service: Rc<S>,
}

fn validate_signature(req: &ServiceRequest) -> Result<(), HttpResponse> {
    let header_map = req.headers();
    tracing::info!("Getting authorization header");
    let signature_header = header_map
        .get("Authorization")
        .ok_or_else(HttpResponse::Unauthorized)?
        .to_str()
        .map_err(|_| HttpResponse::Unauthorized())?;
    let signature_header: SignatureHeader = signature_header
        .parse()
        .map_err(|_| HttpResponse::Unauthorized())?;
    tracing::info!("Got signature header {}", signature_header);

    // Collect headers to sign
    let mut headers_to_sign: HashMap<&str, Vec<&str>> = HashMap::new();
    let mandatory_headers = vec!["(request-target)"];
    for &h in &mandatory_headers {
        let values: Vec<&str> = header_map.get_all(h).map(|h| h.to_str().unwrap()).collect();
        headers_to_sign.insert(h, values);
    }
    let request_target = format!("{} {}", req.method().to_string().to_lowercase(), req.uri());
    headers_to_sign.insert("(request-target)", vec![request_target.as_str()]);

    // Compute the expected signature string
    let signature_string = signature::signature_string(&mandatory_headers, &headers_to_sign);
    tracing::info!("Verifying signature string {}", signature_string);

    // Get the provided signature
    let provided_signature =
        base64::decode(signature_header.signature()).map_err(|_| HttpResponse::BadRequest())?;

    // Load public key associated with the keyId
    let public_key = signature::load_public_key(&format!(
        "./key_repository/{}.pem",
        signature_header.key_id()
    ))
    .map_err(|_| {
        tracing::warn!("Public key does not exist");
        HttpResponse::Unauthorized()
    })?;

    // Decrypt provided signature with client's public key, and make sure it matches the signature string
    let verified = signature::verify(signature_string.as_bytes(), &provided_signature, public_key);
    if verified {
        tracing::info!("Signature validation succeeded");
        Ok(())
    } else {
        tracing::info!("Signature validation failed");
        Err(HttpResponse::Unauthorized().finish())
    }
}

impl<S> Service<ServiceRequest> for SignatureFilterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>>,
    S: 'static,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let validation_result = validate_signature(&req);
        let service = Rc::clone(&self.service);
        Box::pin(async move {
            match validation_result {
                Ok(()) => {
                    let resp = service.call(req).await?;
                    Ok(resp.map_into_boxed_body())
                }
                Err(status) => Ok(req.into_response(status).map_into_boxed_body()),
            }
        })
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug)]
pub struct SignatureFilter;

impl<S> Transform<S, ServiceRequest> for SignatureFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = actix_web::Error>,
    S::Future: 'static,
    S: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = SignatureFilterService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(SignatureFilterService {
            service: Rc::new(service),
        }))
    }
}
