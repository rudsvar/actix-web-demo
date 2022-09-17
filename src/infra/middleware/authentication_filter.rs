//! Middleware adding authentication data to requests.

use crate::infra::security::jwt::Claims;
use actix_http::{
    body::{BoxBody, EitherBody, MessageBody},
    HttpMessage,
};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    HttpResponse,
};
use futures::future::{LocalBoxFuture, Ready};
use std::rc::Rc;

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct AuthenticationFilterService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationFilterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>>,
    S: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            // Continue if authenticated, reject if not
            if req.extensions().get::<Claims>().is_some() {
                svc.call(req).await.map(|r| r.map_into_right_body())
            } else {
                Ok(req
                    .into_response(HttpResponse::Unauthorized())
                    .map_into_left_body())
            }
        })
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug, Default)]
pub struct AuthenticationFilter {}

impl AuthenticationFilter {
    /// Constructs a new [`Authentication`] instance.
    pub fn new() -> Self {
        AuthenticationFilter::default()
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthenticationFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<EitherBody<BoxBody, B>>;
    type Error = S::Error;
    type InitError = ();
    type Transform = AuthenticationFilterService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(AuthenticationFilterService {
            service: Rc::new(service),
        }))
    }
}
