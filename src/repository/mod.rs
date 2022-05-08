//! Repositories for simplifying and standardizing storage access.

use async_trait::async_trait;

pub mod account_repository;

/// Represents a repository - a collection of entities.
/// It provides methods for performing standard CRUD-operations on the entity.
#[async_trait(?Send)]
pub trait Repository<'a> {
    /// A unique identifier for the entity.
    type Id;
    /// Information written to the repository.
    type Store;
    /// Information read from the repository.
    type Load;

    /// Store a new entity in the repository.
    async fn create(&'a mut self, data: Self::Store) -> Self::Load;
    /// Fetch an entity from the repository.
    async fn read(&'a mut self, id: Self::Id) -> Self::Load;
    /// Update an existing entity in the repository.
    async fn update(&'a mut self, data: Self::Store) -> Self::Load;
    /// Delete an existing entity from the repository.
    async fn delete(&'a mut self, id: Self::Id) -> i32;
}
