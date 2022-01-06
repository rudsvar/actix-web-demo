//! A type for representing meta-information in from a request.
//! Used to identify a user and where the request came from.

use actix_http::{header::HeaderMap, StatusCode};
use actix_web::{error::InternalError, web, Error, FromRequest, Responder};
use serde::{Deserialize, Serialize};
use std::{
    future::{self},
    str::FromStr,
};

/// A header is missing.
struct MissingHeaderError<'a>(&'a str);

impl<'a> From<MissingHeaderError<'a>> for InternalError<String> {
    fn from(e: MissingHeaderError<'a>) -> Self {
        InternalError::new(format!("Missing header '{}'", e.0), StatusCode::BAD_REQUEST)
    }
}

/// A header contained non-ascii characters.
struct NonAsciiHeaderError<'a>(&'a str);

impl<'a> From<NonAsciiHeaderError<'a>> for InternalError<String> {
    fn from(e: NonAsciiHeaderError<'a>) -> Self {
        InternalError::new(
            format!("Non-ascii header '{}'", e.0),
            StatusCode::BAD_REQUEST,
        )
    }
}

/// A header cannot be parsed.
struct CannotParseHeaderError<'a>(&'a str);

impl<'a> From<CannotParseHeaderError<'a>> for InternalError<String> {
    fn from(e: CannotParseHeaderError<'a>) -> Self {
        InternalError::new(
            format!("Cannot parse header '{}'", e.0),
            StatusCode::BAD_REQUEST,
        )
    }
}

/// Attempts to retrieve and parse the value of a header.
fn try_parse_header<T>(hm: &HeaderMap, header_name: &str) -> Result<T, InternalError<String>>
where
    T: FromStr,
{
    let header_value = hm.get(header_name).ok_or(MissingHeaderError(header_name))?;
    let header_str = header_value
        .to_str()
        .map_err(|_| NonAsciiHeaderError(header_name))?;
    let parsed = header_str
        .parse()
        .map_err(|_| CannotParseHeaderError(header_name))?;
    Ok(parsed)
}

/// Meta-information about a request.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientContext {
    user_id: usize,
    user_name: String,
    token: String,
}

impl ClientContext {
    /// Constructs a new [`ClientContext`].
    pub fn new(user_id: usize, user_name: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            user_id,
            user_name: user_name.into(),
            token: token.into(),
        }
    }
    fn from_request(req: &actix_web::HttpRequest) -> Result<ClientContext, InternalError<String>> {
        let hm = req.headers();
        let user_id: usize = try_parse_header(hm, "user_id")?;
        let user_name: String = try_parse_header(hm, "user_name")?;
        let token: String = try_parse_header(hm, "token")?;
        let cc = ClientContext::new(user_id, user_name, token);
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

/// Constructs a [`ClientContext`] from a request, logs it, and responds with it.
pub async fn client_context(cc: ClientContext) -> impl Responder {
    tracing::info!("Got request from {:?}", cc);
    web::Json(cc)
}
