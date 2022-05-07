//! Middleware for appending headers to responses.

use actix_http::header::{HeaderName, HeaderValue};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::{
    future::{LocalBoxFuture, Ready},
    FutureExt,
};

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct RequestWrapperService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestWrapperService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        tracing::info!("Request {} {}", req.method(), req.path());
        let fut = self.service.call(req);
        async move {
            let mut res = fut.await?;
            res.headers_mut().append(
                HeaderName::from_static("custom-header"),
                HeaderValue::from_static("custom-value"),
            );
            tracing::info!("Response {}", res.status());
            Ok(res)
        }
        .boxed_local()
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug)]
pub struct RequestWrapper;

impl<S, B> Transform<S, ServiceRequest> for RequestWrapper
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = RequestWrapperService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(RequestWrapperService { service }))
    }
}
