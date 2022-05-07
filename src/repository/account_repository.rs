use super::Repository;
use crate::{
    error::DbError,
    service::account::{Account, NewAccount},
};
use async_trait::async_trait;
use sqlx::PgExecutor;

pub struct AccountRepository<T>(T);

#[async_trait]
impl<'a, T> Repository for AccountRepository<T>
where
    T: Send + Sync,
    &T: PgExecutor<'a>,
{
    type Id = (i32, i32);
    type Store = NewAccount;
    type Load = Result<Account, DbError>;

    async fn fetch<'b>(&'b mut self, id: Self::Id) -> Self::Load {
        sqlx::query_as!(
            Account,
            r#"SELECT * FROM accounts WHERE id = $1 AND owner_id = $2"#,
            id.0,
            id.1,
        )
        .fetch_one(&self.0)
        .await
        .map_err(DbError::from)
    }

    async fn store(&self, data: Self::Store) -> Self::Load {
        todo!()
    }

    async fn update(&self, data: Self::Store) -> Self::Load {
        todo!()
    }

    async fn delete(&self, id: Self::Id) -> i32 {
        todo!()
    }
}

fn foo<'a>(e: impl PgExecutor<'a> + Send + Sync) {
    let ar = AccountRepository(e);
    let account = ar.fetch((0, 0));
}
