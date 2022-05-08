//! Utilities for interacting with the account table.

use super::Repository;
use crate::{
    error::DbError,
    service::account::{Account, NewAccount},
};
use async_trait::async_trait;
use sqlx::PgExecutor;

/// A wrapper for accessing the account table.
#[derive(Copy, Clone, Debug)]
pub struct AccountRepository<T>(pub T);

impl<T> AccountRepository<T> {
    /// Creates a new account repository.
    pub fn new(connection: T) -> Self {
        Self(connection)
    }
}

#[async_trait(?Send)]
impl<'a, T> Repository<'a> for AccountRepository<T>
where
    &'a mut T: PgExecutor<'a> + 'a,
{
    type Id = i32;
    type Store = NewAccount;
    type Load = Result<Account, DbError>;

    async fn create(&'a mut self, new_account: Self::Store) -> Self::Load {
        sqlx::query_as!(
            Account,
            r#"
            INSERT INTO accounts (name, balance, owner_id)
            VALUES ($1, $2, $3)
            RETURNING *"#,
            new_account.name(),
            0i64,
            new_account.owner_id()
        )
        .fetch_one(&mut self.0)
        .await
        .map_err(DbError::from)
    }

    async fn read(&'a mut self, id: Self::Id) -> Self::Load {
        sqlx::query_as!(Account, r#"SELECT * FROM accounts WHERE id = $1"#, id)
            .fetch_one(&mut self.0)
            .await
            .map_err(DbError::from)
    }

    async fn update(&'a mut self, _data: Self::Store) -> Self::Load {
        todo!()
    }

    async fn delete(&'a mut self, _id: Self::Id) -> i32 {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::AccountRepository;
    use crate::{repository::Repository, DbPool};

    #[allow(dead_code)]
    async fn compiles_with_connection(pool: DbPool) {
        let tx = pool.begin().await.unwrap();
        let mut repo = AccountRepository(tx);
        let _ = repo.read(0).await;
    }

    #[allow(dead_code)]
    async fn compiles_with_transaction(pool: DbPool) {
        let conn = pool.acquire().await.unwrap();
        let mut repo = AccountRepository(conn);
        let _ = repo.read(0).await;
    }
}
