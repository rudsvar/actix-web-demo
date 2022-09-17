//! Middleware adding authentication data to requests.

use crate::{
    infra::security::{headers::Auth, jwt::Claims},
    security::jwt,
    DbPool,
};
use actix_http::HttpMessage;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    HttpResponse, ResponseError,
};
use actix_web_grants::permissions::AttachPermissions;
use futures::future::{LocalBoxFuture, Ready};
use std::rc::Rc;
use tracing::Instrument;

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct AuthenticatorService<S> {
    service: Rc<S>,
}

async fn authenticate(req: &ServiceRequest) -> Result<Claims, HttpResponse> {
    // Get db pool
    let db = req
        .app_data::<Data<DbPool>>()
        .ok_or_else(|| HttpResponse::InternalServerError().finish())?;
    // Extract header
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| HttpResponse::Unauthorized().finish())?;

    // Convert header to str
    let auth_header = auth_header
        .to_str()
        .map_err(|_| HttpResponse::BadRequest().finish())?;
    // Parse auth data
    let auth = Auth::from_header(auth_header).ok_or_else(|| HttpResponse::BadRequest().finish())?;

    // Handle the auth methods
    let claims = match auth {
        Auth::Basic(basic_auth) => {
            jwt::create_claims(db, basic_auth.username(), basic_auth.password())
                .await
                .map_err(|e| e.error_response())?
        }
        Auth::Bearer(bearer_auth) => {
            jwt::decode_jwt(bearer_auth.token()).map_err(|e| e.error_response())?
        }
    };

    Ok(claims)
}

impl<S, B> Service<ServiceRequest> for AuthenticatorService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>>,
    S: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            // Attach claims to request
            let resp = if let Ok(claims) = authenticate(&req).await {
                let span = tracing::info_span!("principal", principal = claims.id());
                req.attach(claims.roles().to_vec());
                req.extensions_mut().insert(claims);
                svc.call(req).instrument(span).await?
            } else {
                svc.call(req).await?
            };

            Ok(resp)
        })
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug, Default)]
pub struct Authenticator {}

impl Authenticator {
    /// Constructs a new [`Authentication`] instance.
    pub fn new() -> Self {
        Authenticator::default()
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authenticator
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type InitError = ();
    type Transform = AuthenticatorService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(AuthenticatorService {
            service: Rc::new(service),
        }))
    }
}
