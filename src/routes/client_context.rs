use std::future::{self};

use actix_http::StatusCode;
use actix_web::{error::InternalError, Error, FromRequest, HttpResponse, Responder};

#[derive(Debug)]
pub struct ClientContext {
    user_id: usize,
    user_name: String,
    token: String,
}

struct MissingHeader<'a>(&'a str);

impl<'a> From<MissingHeader<'a>> for InternalError<String> {
    fn from(mh: MissingHeader) -> InternalError<String> {
        InternalError::new(format!("Missing header {}", mh.0), StatusCode::BAD_REQUEST)
    }
}

struct InvalidHeader<'a>(&'a str);

impl<'a> From<InvalidHeader<'a>> for InternalError<String> {
    fn from(ih: InvalidHeader) -> InternalError<String> {
        InternalError::new(format!("Invalid header {}", ih.0), StatusCode::BAD_REQUEST)
    }
}

impl ClientContext {
    pub fn new(user_id: usize, user_name: String, token: String) -> Self {
        Self {
            user_id,
            user_name,
            token,
        }
    }
    fn from_request(req: &actix_web::HttpRequest) -> Result<ClientContext, InternalError<String>> {
        let headers = req.headers();
        let user_id = headers
            .get("user_id")
            .ok_or(MissingHeader("user_id"))?
            .to_str()
            .unwrap();
        let user_name = headers
            .get("user_name")
            .ok_or(MissingHeader("user_name"))?
            .to_str()
            .unwrap();
        let token = headers
            .get("token")
            .ok_or(MissingHeader("token"))?
            .to_str()
            .unwrap();
        let cc = ClientContext::new(
            user_id.parse().map_err(|_| InvalidHeader("user_id"))?,
            user_name.to_string(),
            token.to_string(),
        );
        Ok(cc)
    }
}

impl FromRequest for ClientContext {
    type Error = Error;
    type Future = future::Ready<Result<Self, Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future {
        future::ready(ClientContext::from_request(req).map_err(|e| e.into()))
    }
}

pub async fn client_context(cc: ClientContext) -> impl Responder {
    log::info!("Got request from {:?}", cc);
    HttpResponse::Ok().body(format!("{:?}", cc))
}
