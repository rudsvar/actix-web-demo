//! An API for transferring money between accounts.

use crate::{
    error::DbError,
    service::{
        transfer::{transfer_model::NewTransfer, transfer_repository},
        AppResult,
    },
    DbPool,
};
use actix_web::{web, HttpResponse};

#[actix_web::post("/transfers")]
pub async fn create_transfer(
    db: web::Data<DbPool>,
    new_transfer: web::Json<NewTransfer>,
) -> AppResult<HttpResponse> {
    let mut tx = db.get_ref().begin().await.map_err(DbError::from)?;
    let transfer = transfer_repository::insert_transfer(&mut tx, new_transfer.into_inner()).await?;
    tx.commit().await.map_err(DbError::from)?;
    Ok(HttpResponse::Created().json(transfer))
}
