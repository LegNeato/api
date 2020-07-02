//! Actix + Juniper + Tokio-Postgres
//!
//! Juniper with actix-web
use std::io;
use std::sync::Arc;

use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use tokio_postgres::Client;
use uuid::Uuid;

mod context;
mod db;
mod schema;
mod twig;
mod utils;

use crate::schema::{create_schema, Schema};

async fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:8080/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<AppState>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let res = data.execute(
            &st.st,
            &context::GraphQLContext {
                pool: Arc::clone(&st.pool),
            },
        );
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

// TODO: use this struct
pub struct OngoingUpload {
    packageName: String,
    done: bool,
}

impl actix_web::error::ResponseError for &str {}

async fn upload_package(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    let mut fields = HashMap::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let mime_type = field.content_type();
        println!("{}", mime_type.type_());
        let filename = content_type.get_filename();
        let unqiueName = Uuid::new_v4().to_simple().to_string();
        let filepath = format!("tmp/{}", filename.unwrap_or("none"));
        // File::create is blocking operation, use threadpool
        let mut f = web::block(move || std::fs::File::create(Path::new(&filepath)))
            .await
            .unwrap();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            match filename {
                None => {
                    println!("{:?}", content_type.get_name());
                    fields.insert(content_type.get_name().unwrap(), data);
                }
                Some(n) => {
                    // filesystem operations are blocking, we have to use threadpool
                    f = web::block(move || f.write_all(&data).map(|_| f)).await?;
                }
            }
        }
    }
    let apiKey = fields.get(&"apiKey").ok_or("invalid apikey")?;
    Ok(HttpResponse::Ok().into())
}

async fn index(
    st: web::Data<AppState>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Welcome to Nest.land's Rust API"))
}

pub struct AppState {
    pool: Arc<Client>,
    st: Arc<Schema>,
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let client = db::connect().await.unwrap();
    // Create Juniper schema
    let schema = std::sync::Arc::new(create_schema());
    let conn = std::sync::Arc::new(client);
    // Start http server
    HttpServer::new(move || {
        App::new()
            .wrap(Cors::new().supports_credentials().finish())
            .data(AppState {
                st: schema.clone(),
                pool: conn.clone(),
            })
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
            .service(web::resource("/package").route(web::post().to(upload_package)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
