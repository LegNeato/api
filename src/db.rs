// Postgres database management for Nest API

use crate::schema::{NewUser, Package, User, NewPackage};
use crate::utils::{create_api_key, first, normalize};
use chrono::{DateTime, Utc};
use dotenv;
use postgres_array::array::Array;
use std::sync::Arc;
use std::time::SystemTime;
use tokio_postgres::{Client, Error, NoTls};

// establish connection with Postgres db
pub async fn connect() -> Result<Client, Error> {
    let host = dotenv::var("DB_HOST").unwrap_or("localhost".to_string());
    let user = dotenv::var("DB_USER").unwrap_or("nest".to_string());
    let database_name = dotenv::var("DB_NAME").unwrap_or("nest".to_string());
    let pass = dotenv::var("DB_PASS").unwrap_or("123".to_string());
    let (client, connection) = tokio_postgres::connect(
        &format!(
            "host={} user={} dbname={} password={}",
            host, user, database_name, pass
        ),
        NoTls,
    )
    .await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

// Method to retrieve a package from db
pub async fn get_package(db: Arc<Client>, name: String) -> Result<Package, String> {
    let rows = &db
        .query("SELECT * FROM packages WHERE name = $1", &[&name])
        .await
        .unwrap();
    let _row = first(rows);
    if let Some(x) = _row {
        let row = _row.unwrap();
        let upload_names: Array<String> = row.get(7);
        Ok(Package {
            name: row.get(0),
            normalized_name: row.get(1),
            owner: row.get(2),
            description: row.get(3),
            repository: row.get(4),
            latest_version: row.get(5),
            latest_stable_version: row.get(6),
            package_upload_names: upload_names.iter().cloned().collect(),
            locked: row.get(8),
            malicious: row.get(9),
            unlisted: row.get(10),
            updated_at: format!("{:?}", row.get::<usize, DateTime<Utc>>(11)),
            created_at: format!("{:?}", row.get::<usize, DateTime<Utc>>(12)),
        })
    } else {
        Err("Not found".to_string())
    }
}

// Method to retrieve a user from db
pub async fn get_user_by_key(db: Arc<Client>, api_key: String) -> Result<User, String> {
    let rows = &db
        .query("SELECT * FROM users WHERE api_key = $1", &[&api_key])
        .await
        .unwrap();
    let _row = first(rows);
    if let Some(x) = _row {
        let row = _row.unwrap();
        let package_names: Array<String> = row.get(4);
        Ok(User {
            name: row.get(0),
            normalized_name: row.get(1),
            api_key: row.get(3),
            package_names: package_names.iter().cloned().collect(),
            created_at: format!("{:?}", row.get::<usize, DateTime<Utc>>(5)),
        })
    } else {
        Err("Not found".to_string())
    }
}

// Method to create a user
pub async fn create_user(db: Arc<Client>, new_user: NewUser) -> Result<User, Error> {
    let api_key = create_api_key();
    let normalized_name = normalize(&new_user.name);
    let rows = &db
        .query("INSERT INTO users (name, normalized_name, password, api_key, package_names, created_at) VALUES ($1, $2, $3, $4, $5, $6)", &[&new_user.name, &normalized_name, &new_user.password, &api_key, &Array::<String>::from_vec(vec![], 0), &Utc::now()])
        .await?;
    let name = new_user.name;
    Ok(User {
        name: name,
        normalized_name: normalized_name,
        api_key: api_key,
        package_names: vec![],
        created_at: format!("{:?}", Utc::now()),
    })
}

// TODO: implement upload creation
pub async fn create_package_uploads(db: Arc<Client>, package: NewPackage) -> Result<(), Error> {
    Ok(())
}
