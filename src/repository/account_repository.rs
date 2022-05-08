use super::Repository;
use crate::{
    error::DbError,
    service::account::{Account, NewAccount},
    DbPool,
};
use async_trait::async_trait;
use sqlx::{PgExecutor, PgPool, Postgres, Transaction};

pub struct AccountRepository(PgPool);

#[async_trait]
impl Repository for AccountRepository {
    type Id = (i32, i32);
    type Store = NewAccount;
    type Load = Result<Account, DbError>;

    async fn fetch<'b>(&'b self, id: Self::Id) -> Self::Load {
        let result = sqlx::query_as!(
            Account,
            r#"SELECT * FROM accounts WHERE id = $1 AND owner_id = $2"#,
            id.0,
            id.1,
        )
        .fetch_one(&self.0)
        .await
        .map_err(DbError::from)?;
        Ok(result)
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

async fn foo<'a>(e: impl PgExecutor<'static>) {
    sqlx::query("select * from users").execute(e).await;
    let x: DbPool = todo!();
    let tx: Transaction<'_, Postgres> = x.begin().await.unwrap();
    foo(&mut tx).await;
    // let ar = AccountRepository(e);
    // let account = ar.fetch((0, 0));
}
