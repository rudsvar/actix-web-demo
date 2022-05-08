use async_trait::async_trait;

pub mod account_repository;

#[async_trait]
pub trait Repository {
    type Id;
    type Store;
    type Load;

    async fn fetch(&self, id: Self::Id) -> Self::Load;
    async fn store(&self, data: Self::Store) -> Self::Load;
    async fn update(&self, data: Self::Store) -> Self::Load;
    async fn delete(&self, id: Self::Id) -> i32;
}
