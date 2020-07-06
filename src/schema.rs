//! Juniper GraphQL handling done here
use crate::context::GraphQLContext;
use crate::db::{
    create_user, get_modules, get_package, get_users, get_user_by_key, publish_package, get_user_by_name,
};
use juniper::FieldResult;
use juniper::RootNode;
use juniper::{GraphQLInputObject, GraphQLObject, EmptySubscription, FieldError};
use std::sync::Arc;
use tokio::runtime::Runtime;
use futures::{Stream, StreamExt};
use std::pin::Pin;

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
#[derive(GraphQLObject)]
#[graphql(description = "A nest.land package author [restricted]")]
pub struct PublicUser {
    pub name: String,
    pub normalized_name: String,
    pub package_names: Vec<String>,
    pub created_at: String,
}

// Define graphql schema for NewPackage
#[derive(GraphQLInputObject)]
#[graphql(description = "A nest.land package upload")]
pub struct NewPackageUpload {
    pub name: String,
    pub api_key: String,
    pub description: String,
    pub repository: String,
    pub upload: bool,
    pub entry: String,
    pub stable: bool,
    pub unlisted: bool,
    pub version: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A nest.land package")]
pub struct NewPackage {
    pub name: String,
    pub api_key: String,
    pub description: String,
    pub repository: String,
    pub locked: bool,
    pub malicious: bool,
    pub unlisted: bool,
}

// Define graphql schema for NewPackage
#[derive(GraphQLInputObject)]
#[graphql(description = "A nest.land new user")]
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
#[juniper::graphql_object(Context = GraphQLContext)]
impl QueryRoot {
    async fn modules(ctx: &GraphQLContext) -> FieldResult<Vec<Package>> {
        Ok(get_modules(Arc::clone(&ctx.pool)).await?)
    }
    async fn package(ctx: &GraphQLContext, name: String) -> FieldResult<Package> {
        Ok(get_package(Arc::clone(&ctx.pool), name).await?)
    }
    async fn user_by_name(ctx: &GraphQLContext, name: String) -> FieldResult<PublicUser> {
        Ok(get_user_by_name(Arc::clone(&ctx.pool), name).await?)
    }
    fn users(ctx: &GraphQLContext) -> FieldResult<Vec<PublicUser>> {
        Ok(get_users(Arc::clone(&ctx.pool)).await?)
    }
    async fn user(ctx: &GraphQLContext, api_key: String) -> FieldResult<User> {
        Ok(get_user_by_key(Arc::clone(&ctx.pool), api_key).await?)
    }
}

pub struct MutationRoot;

// Define MutationRoot for GraphQL
#[juniper::graphql_object(Context = GraphQLContext)]
impl MutationRoot {
    async fn create_user(ctx: &GraphQLContext, new_user: NewUser) -> FieldResult<User> {
        Ok(create_user(Arc::clone(&ctx.pool), new_user).await?)
    }
    async fn create_package(
        ctx: &GraphQLContext,
        new_package: NewPackage,
    ) -> FieldResult<NewPackageResult> {
        Ok(publish_package(Arc::clone(&ctx.pool), new_package).await?)
    }
}

pub struct Subscription;

type StringStream = Pin<Box<dyn Stream<Item = Result<String, FieldError>> + Send>>;

#[juniper::graphql_subscription(Context = GraphQLContext)]
impl Subscription {
    async fn hello_world() -> StringStream {
        let stream = tokio::stream::iter(vec![
            Ok(String::from("Hello")),
            Ok(String::from("World!"))
        ]);
        Box::pin(stream)
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, Subscription>;

// Expose create schema method
pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, Subscription {})
}
