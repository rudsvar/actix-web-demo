use actix_http::header::{HeaderName, HeaderValue};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::{
    future::{LocalBoxFuture, Ready},
    FutureExt,
};

pub struct ResponseAppenderService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ResponseAppenderService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        async move {
            let mut res = fut.await?;
            res.headers_mut().append(
                HeaderName::from_static("custom-header"),
                HeaderValue::from_static("custom-value"),
            );
            Ok(res)
        }
        .boxed_local()
    }
}

pub struct ResponseAppender;

impl<S, B> Transform<S, ServiceRequest> for ResponseAppender
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = ResponseAppenderService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(ResponseAppenderService { service }))
    }
}
