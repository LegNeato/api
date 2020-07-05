//! Juniper GraphQL handling done here
use crate::context::GraphQLContext;
use crate::db::{create_user, get_package, get_user_by_key};
use juniper::FieldResult;
use juniper::RootNode;
use juniper::{GraphQLInputObject, GraphQLObject};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio_postgres::Client;

// Define GraphQL schema for package retrival
#[derive(GraphQLObject)]
#[graphql(description = "A nest.land package")]
pub struct Package {
    pub name: String,
    pub normalized_name: String,
    pub owner: String,
    pub description: String,
    pub repository: String,
    pub latest_version: String,
    pub latest_stable_version: String,
    pub package_upload_names: Vec<String>,
    pub locked: bool,
    pub malicious: bool,
    pub unlisted: bool,
    pub updated_at: String,
    pub created_at: String,
}

// Define GraphQL schema for User retrival
#[derive(GraphQLObject)]
#[graphql(description = "A nest.land package author")]
pub struct User {
    pub name: String,
    pub normalized_name: String,
    pub api_key: String,
    pub package_names: Vec<String>,
    pub created_at: String,
}

// Define graphql schema for NewPackage
#[derive(GraphQLInputObject)]
#[graphql(description = "A nest.land package")]
pub struct NewPackage {
    pub name: String,
    pub owner: String,
    pub description: String,
    pub repository: String,
    pub latest_version: String,
    pub latest_stable_version: String,
    pub package_upload_names: Vec<String>,
    pub locked: bool,
    pub malicious: bool,
    pub unlisted: bool,
}

// Define graphql schema for NewPackage
#[derive(GraphQLInputObject)]
#[graphql(description = "A nest.land package")]
pub struct NewUser {
    pub name: String,
    pub password: String,
}

pub struct QueryRoot;

// Define QueryRoot for GraphQL
#[juniper::object(Context = GraphQLContext)]
impl QueryRoot {
    fn package(ctx: &GraphQLContext, name: String) -> FieldResult<Package> {
        Ok(Runtime::new()
            .unwrap()
            .block_on(get_package(Arc::clone(&ctx.pool), name))?)
    }
    fn user(ctx: &GraphQLContext, api_key: String) -> FieldResult<User> {
        Ok(Runtime::new()
            .unwrap()
            .block_on(get_user_by_key(Arc::clone(&ctx.pool), api_key))?)
    }
}

pub struct MutationRoot;

// Define MutationRoot for GraphQL
#[juniper::object(Context = GraphQLContext)]
impl MutationRoot {
    fn create_user(ctx: &GraphQLContext, new_user: NewUser) -> FieldResult<User> {
        Ok(Runtime::new()
            .unwrap()
            .block_on(create_user(Arc::clone(&ctx.pool), new_user))?)
    }
    fn create_package(ctx: &GraphQLContext, new_package: NewPackage) -> FieldResult<Package> {
        Ok(Package {
            name: new_package.name.to_owned(),
            normalized_name: new_package.name.to_owned(),
            owner: new_package.owner.to_owned(),
            description: new_package.description.to_owned(),
            repository: new_package.repository.to_owned(),
            latest_version: new_package.latest_version.to_owned(),
            latest_stable_version: new_package.latest_stable_version.to_owned(),
            package_upload_names: new_package.package_upload_names,
            locked: false,
            malicious: false,
            unlisted: false,
            updated_at: "sometime".to_owned(),
            created_at: "sometime".to_owned(),
        })
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

// Expose create schema method
pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
