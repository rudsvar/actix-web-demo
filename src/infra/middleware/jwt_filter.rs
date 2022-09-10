//! Middleware for appending headers to responses.

use std::rc::Rc;

use actix_http::{body::BoxBody, HttpMessage};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    HttpResponse, ResponseError,
};
use futures::future::{LocalBoxFuture, Ready};
use tracing::Instrument;

use crate::infra::security::jwt::Claims;

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct PrincipalInitService<S> {
    service: Rc<S>,
}

fn claims_from_request(req: &ServiceRequest) -> Result<Claims, HttpResponse> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| HttpResponse::Unauthorized().finish())?;

    let auth_header = auth_header
        .to_str()
        .map_err(|_| HttpResponse::BadRequest().finish())?;

    let token = auth_header.trim_start_matches("Bearer ");
    let claims = crate::security::jwt::decode_jwt(token).map_err(|e| e.error_response())?;

    Ok(claims)
}

impl<S> Service<ServiceRequest> for PrincipalInitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>>,
    S: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            let claims = match claims_from_request(&req) {
                Ok(claims) => claims,
                Err(e) => return Ok(req.into_response(e).map_into_boxed_body()),
            };
            let span = tracing::info_span!("principal", principal = claims.id());
            req.extensions_mut().insert(claims);

            let resp = svc.call(req).instrument(span).await?;
            Ok(resp.map_into_boxed_body())
        })
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug)]
pub struct PrincipalInit;

impl<S> Transform<S, ServiceRequest> for PrincipalInit
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = actix_web::Error>,
    S: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = PrincipalInitService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(PrincipalInitService {
            service: Rc::new(service),
        }))
    }
}
