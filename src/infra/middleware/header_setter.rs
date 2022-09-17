//! Middleware adding authentication data to requests.

use actix_http::header::HeaderName;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures::future::{LocalBoxFuture, Ready};
use http::HeaderValue;
use std::rc::Rc;
use tracing::Instrument;

/// A service for appending headers to responses.
#[derive(Debug)]
pub struct HeaderSetterService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for HeaderSetterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>>,
    S: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            tracing::debug!("Setting default headers");

            let headers_mut = req.headers_mut();
            // Set request id if not set
            if !headers_mut.contains_key("request-id") {
                let request_id = HeaderName::from_static("request-id");
                let uuid = HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()).unwrap();
                headers_mut.insert(request_id, uuid);
            }
            let rid = headers_mut.get("request-id").expect("just inserted it");
            let rid = rid.to_str().unwrap().to_string();

            // Start request
            let span = tracing::info_span!("request", request_id = &rid);
            let res = svc.call(req).instrument(span).await;

            // Append request id to response
            match res {
                Ok(mut res) => {
                    let request_id = HeaderName::from_static("request-id");
                    let uuid = HeaderValue::from_str(&rid).unwrap();
                    res.headers_mut().append(request_id, uuid);
                    Ok(res)
                }
                Err(e) => Err(e),
            }
        })
    }
}

/// Middleware for appending headers to responses.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeaderSetter {}

impl HeaderSetter {
    /// Constructs a new [`Authentication`] instance.
    pub fn new() -> Self {
        HeaderSetter::default()
    }
}

impl<S, B> Transform<S, ServiceRequest> for HeaderSetter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = S::Error;
    type InitError = ();
    type Transform = HeaderSetterService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ready(Ok(HeaderSetterService {
            service: Rc::new(service),
        }))
    }
}
