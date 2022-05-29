//! Middleware for appending headers to responses.

use crate::security::signature::{self, Headers, SignatureHeader};
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
use std::rc::Rc;

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct SignatureFilterService<S> {
    service: Rc<S>,
}

/// Extracts the request body.
async fn has_body(req: &mut ServiceRequest) -> bool {
    let mut body = BytesMut::new();
    let mut stream = req.take_payload();

    while let Some(chunk) = stream.next().await {
        body.extend_from_slice(&chunk.unwrap());
    }

    let has_body = !body.is_empty();

    // Reset payload
    let (_, mut payload) = Payload::create(true);
    payload.unread_data(body.into());
    req.set_payload(payload.into());

    has_body
}

async fn validate_signature(req: &mut ServiceRequest) -> Result<(), HttpResponse> {
    let has_body = has_body(req).await;
    let header_map = req.headers();

    // Extract signature information
    tracing::info!("Getting authorization header");
    let signature_header = header_map
        .get("Authorization")
        .ok_or_else(HttpResponse::Unauthorized)?
        .to_str()
        .map_err(|_| HttpResponse::BadRequest())?;
    let signature_header: SignatureHeader = signature_header
        .parse()
        .map_err(|_| HttpResponse::BadRequest())?;
    tracing::info!("Got signature header {}", signature_header);

    // Check for missing signature headers
    let mut mandatory_headers = vec!["(request-target)", "date"];
    if has_body {
        tracing::info!("Request has body, adding digest to mandatory headers");
        mandatory_headers.push("digest");
    }
    let missing_headers: Vec<&str> = mandatory_headers
        .into_iter()
        .filter(|h| !signature_header.headers().contains(&h.to_string()))
        .collect();
    if !missing_headers.is_empty() {
        tracing::warn!("Missing mandatory signature headers {:?}", missing_headers);
        return Err(HttpResponse::Unauthorized().finish());
    }

    // Collect headers to sign
    let signed_headers = signature_header.headers();
    let mut headers_to_sign = Headers::new();
    let request_target = format!("{} {}", req.method().as_str().to_lowercase(), req.uri());
    for header in signed_headers.iter() {
        match header.as_str() {
            // Compute request-target
            "(request-target)" => {
                headers_to_sign.add(header, &request_target);
            }
            // Append header
            header => {
                for value in header_map.get_all(header) {
                    headers_to_sign.add(header, value.to_str().unwrap());
                }
            }
        }
    }

    tracing::info!("Headers to sign: {:?}", headers_to_sign);

    // Compute the expected signature string
    let signature_string = headers_to_sign.signature_string();
    tracing::info!("Verifying signature string {}", signature_string);

    // Get the provided signature
    let provided_signature =
        base64::decode(signature_header.signature()).map_err(|_| HttpResponse::BadRequest())?;

    // Load public key associated with the keyId
    let public_key = signature::load_public_key(
        &format!("./key_repository/{}.pem", signature_header.key_id()),
        signature_header.algorithm(),
    )
    .map_err(|_| {
        tracing::warn!("Public key does not exist");
        HttpResponse::Unauthorized()
    })?;

    // Decrypt provided signature with client's public key, and make sure it matches the signature string
    let verified = signature::verify(signature_string.as_bytes(), &provided_signature, public_key)
        .map_err(|_| HttpResponse::Unauthorized())?;
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

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        Box::pin(async move {
            let validation_result = validate_signature(&mut req).await;
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
