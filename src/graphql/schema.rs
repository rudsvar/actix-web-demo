//! GraphQL schemas.

use crate::DbPool;
use juniper::FieldResult;
use juniper::{EmptySubscription, RootNode};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[derive(GraphQLObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct Human {
    id: String,
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct NewHuman {
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

/// The root of all queries.
#[derive(Clone, Debug)]
pub struct QueryRoot(DbPool);

#[juniper::graphql_object]
impl QueryRoot {
    fn human(_id: String) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_owned(),
            name: "Luke".to_owned(),
            appears_in: vec![Episode::NewHope],
            home_planet: "Mars".to_owned(),
        })
    }
}

/// The root of all mutations.
#[derive(Clone, Copy, Debug)]
pub struct MutationRoot;

#[juniper::graphql_object]
impl MutationRoot {
    fn create_human(new_human: NewHuman) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_owned(),
            name: new_human.name,
            appears_in: new_human.appears_in,
            home_planet: new_human.home_planet,
        })
    }
}

/// The GraphQL schema.
pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

/// Creates the main schema.
pub fn create_schema(pool: DbPool) -> Schema {
    Schema::new(QueryRoot(pool), MutationRoot, EmptySubscription::new())
}
