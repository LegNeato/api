//! Juniper GraphQL handling done here
use crate::context::GraphQLContext;
use crate::db::{create_user, get_package, get_user_by_key, create_package_uploads};
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
    pub normalizedName: String,
    pub owner: String,
    pub description: String,
    pub repository: String,
    pub latestVersion: String,
    pub latestStableVersion: String,
    pub packageUploadNames: Vec<String>,
    pub locked: bool,
    pub malicious: bool,
    pub unlisted: bool,
    pub updatedAt: String,
    pub createdAt: String,
}

// Define GraphQL schema for User retrival
#[derive(GraphQLObject)]
#[graphql(description = "A nest.land package author")]
pub struct User {
    pub name: String,
    pub normalizedName: String,
    pub apiKey: String,
    pub packageNames: Vec<String>,
    pub createdAt: String,
}

// Define graphql schema for NewPackage
#[derive(GraphQLInputObject)]
#[graphql(description = "A nest.land package")]
pub struct NewPackage {
    pub name: String,
    pub apiKey: String,
    pub description: String,
    pub repository: String,
    pub upload: bool,
    pub entry: String,
    pub stable: bool,
    pub unlisted: bool,
    pub version: String,
}

// Define graphql schema for NewPackage
#[derive(GraphQLInputObject)]
#[graphql(description = "A nest.land package")]
pub struct NewUser {
    pub name: String,
    pub password: String,
}

#[derive(GraphQLObject)]
#[graphql(description = "Package upload result")]
pub struct NewPackageResult {
    pub ok: bool,
    pub msg: String,
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
    fn user(ctx: &GraphQLContext, apiKey: String) -> FieldResult<User> {
        Ok(Runtime::new()
            .unwrap()
            .block_on(get_user_by_key(Arc::clone(&ctx.pool), apiKey))?)
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
    fn create_package(ctx: &GraphQLContext, new_package: NewPackage) -> FieldResult<NewPackageResult> {
        Ok(Runtime::new()
            .unwrap()
            .block_on(create_package_uploads(Arc::clone(&ctx.pool), new_package))?)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

// Expose create schema method
pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
