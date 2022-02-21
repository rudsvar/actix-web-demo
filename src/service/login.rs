//! A service that can receive user information and validate it.

use actix_http::StatusCode;
use actix_web::{error::InternalError, web, Error, FromRequest, HttpResponse};
use futures::{future, FutureExt};
use sqlx::PgPool;
use uuid::Uuid;

/// A guarantee that the credentials of this user have been verified.
/// This type can only be created from a request with the appropriate credentials.
#[derive(Copy, Clone, Debug)]
pub struct AuthenticatedUser {
    id: Uuid,
}

impl AuthenticatedUser {
    /// Returns the id of the authenticated user.
    pub fn id(&self) -> &Uuid {
        &self.id
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = future::LocalBoxFuture<'static, Result<Self, Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_http::Payload) -> Self::Future {
        // Get database connection
        let conn = match req.app_data::<web::Data<PgPool>>() {
            Some(conn) => conn.get_ref().clone(),
            None => {
                return async {
                    Err(InternalError::new(
                        "Can't connect to database",
                        StatusCode::SERVICE_UNAVAILABLE,
                    )
                    .into())
                }
                .boxed_local()
            }
        };

        // Extract headers
        let headers = req.headers();
        let id = headers.get("id").expect("id header").to_str().unwrap();
        let id = Uuid::parse_str(id).expect("valid id");
        let password = headers
            .get("password")
            .expect("password header")
            .to_str()
            .unwrap()
            .to_owned();

        async move {
            let is_valid_result = crate::db::user::verify_password(&conn, &id, &password).await;
            match is_valid_result {
                Ok(is_valid) => {
                    if is_valid {
                        Ok(AuthenticatedUser { id })
                    } else {
                        Err(InternalError::new("Unauthorized", StatusCode::UNAUTHORIZED).into())
                    }
                }
                Err(e) => {
                    Err(InternalError::new(e.to_string(), StatusCode::SERVICE_UNAVAILABLE).into())
                }
            }
        }
        .boxed_local()
    }
}

#[actix_web::get("/login")]
pub async fn login(user: AuthenticatedUser) -> HttpResponse {
    tracing::info!("Successfully validated {}", user.id);
    HttpResponse::Ok().finish()
}
