//! Account service.

use crate::infra::error::DbError;
use crate::model::account_model::Account;
use crate::repository::account_repository;
use crate::DbPool;
use generated::account_service_server::AccountService;
use generated::{AccountRequest, AccountResponse};
use tonic::{Request, Response, Status};

/// Generated types for the echo service.
#[allow(missing_copy_implementations)]
pub mod generated {
    tonic::include_proto!("account");
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        AccountResponse {
            id: account.id,
            name: account.name,
            balance: account.balance,
            owner_id: account.owner_id,
        }
    }
}

/// The account service implementation.
#[derive(Clone, Debug)]
pub struct AccountServiceImpl {
    db: DbPool,
}

impl AccountServiceImpl {
    /// Constructs a new [`AccountService`].
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl AccountService for AccountServiceImpl {
    async fn get_account(
        &self,
        req: Request<AccountRequest>,
    ) -> Result<Response<AccountResponse>, Status> {
        let req = req.into_inner();

        // Start transaction
        let mut tx = self.db.begin().await.map_err(DbError::from)?;

        // Fetch account
        let account =
            account_repository::fetch_account(&mut tx, req.user_id, req.account_id).await?;

        // End transaction
        tx.commit().await.map_err(DbError::from)?;

        Ok(Response::new(account.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        grpc::account::{
            generated::{account_service_server::AccountService, AccountRequest, AccountResponse},
            AccountServiceImpl,
        },
        DbPool,
    };
    use tonic::{Code, Request};

    #[sqlx::test]
    async fn get_existing_account_works(db: DbPool) {
        let service = AccountServiceImpl::new(db);
        let request = AccountRequest {
            user_id: 1,
            account_id: 1,
        };
        let response = service.get_account(Request::new(request)).await.unwrap();
        let account_response = response.into_inner();
        assert_eq!(
            AccountResponse {
                id: 1,
                name: "acc1".into(),
                balance: 100,
                owner_id: 1,
            },
            account_response
        );
    }

    #[sqlx::test]
    async fn get_non_existing_account_fails(db: DbPool) {
        let service = AccountServiceImpl::new(db);
        let request = AccountRequest {
            user_id: 0,
            account_id: 0,
        };
        let status = service
            .get_account(Request::new(request))
            .await
            .err()
            .unwrap();
        assert_eq!(Code::NotFound, status.code());
    }
}
