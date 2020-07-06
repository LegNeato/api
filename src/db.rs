// Postgres database management for Nest API

use crate::schema::{NewPackage, NewPackageResult, NewPackageUpload, NewUser, Package, PublicUser, User};
use crate::utils::{create_api_key, first, normalize};
use chrono::{DateTime, Utc};
use dotenv;
use postgres_array::array::Array;
use std::sync::Arc;
use tokio_postgres::{Client, Error, NoTls};
use serde::{Deserialize, Serialize};
use postgres_types::Json;
use postgres_types::{FromSql};

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

// Method to retrieve all modules from db
pub async fn get_modules(db: Arc<Client>) -> Result<Vec<Package>, String> {
    let rows = &db
        .query("SELECT * FROM packages", &[])
        .await
        .unwrap();
    let mut modules: Vec<Package> = Vec::new();
    for i in 0..rows.len() {
        let row = &rows[i];
        let upload_names: Array<String> = row.get(7);
        modules.push(Package {
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
        });
    }
    Ok(modules)
}

// Method to retrieve a package from db
pub async fn get_package(db: Arc<Client>, name: String) -> Result<Package, String> {
    let rows = &db
        .query("SELECT * FROM packages WHERE name = $1", &[&name])
        .await
        .unwrap();
    let _row = first(rows);
    if let Some(_) = _row {
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

// Method to retrieve all users from db
pub async fn get_users(db: Arc<Client>) -> Result<Vec<PublicUser>, String> {
    let rows = &db
        .query("SELECT * FROM users", &[])
        .await
        .unwrap();
    let mut users: Vec<PublicUser> = Vec::new();
    for i in 0..rows.len() {
        let row = &rows[i];
        let package_names: Array<String> = row.get(4);
        users.push(PublicUser {
            name: row.get(0),
            normalized_name: row.get(1),
            package_names: package_names.iter().cloned().collect(),
            created_at: format!("{:?}", row.get::<usize, DateTime<Utc>>(5)),
        });
    }
    Ok(users)
}

// Method to retrieve a user from db using name
pub async fn get_user_by_name(db: Arc<Client>, name: String) -> Result<PublicUser, String> {
    let rows = &db
        .query("SELECT * FROM users WHERE name = $1", &[&name])
        .await
        .unwrap();
    let _row = first(rows);
    if let Some(_) = _row {
        let row = _row.unwrap();
        let package_names: Array<String> = row.get(4);
        Ok(PublicUser {
            name: row.get(0),
            normalized_name: row.get(1),
            package_names: package_names.iter().cloned().collect(),
            created_at: format!("{:?}", row.get::<usize, DateTime<Utc>>(5)),
        })
    } else {
        Err("Not found".to_string())
    }
}

// Method to retrieve a user from db using API key
pub async fn get_user_by_key(db: Arc<Client>, api_key: String) -> Result<User, String> {
    let rows = &db
        .query("SELECT * FROM users WHERE apiKey = $1", &[&api_key])
        .await
        .unwrap();
    let _row = first(rows);
    if let Some(_) = _row {
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
    let curr_time = Utc::now();
    let normalized_name = normalize(&new_user.name);
    let _ = &db
        .query("INSERT INTO users (name, normalizedName, password, apiKey, packageNames, createdAt) VALUES ($1, $2, $3, $4, $5, $6)", &[&new_user.name, &normalized_name, &new_user.password, &api_key, &Array::<String>::from_vec(vec![], 0), &curr_time])
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

// TODO: publish packages
pub async fn publish_package(
    db: Arc<Client>,
    package: NewPackage,
) -> Result<NewPackageResult, Error> {
    let user_package_rows = &db
        .query(
            "SELECT * FROM users WHERE apiKey = $1 AND $2 = ANY(packageNames)",
            &[&package.api_key, &package.name],
        )
        .await?;
    let rows = &db
        .query("SELECT * FROM packages WHERE name = $1", &[&package.name])
        .await?;
    let normalized_name = normalize(&package.name);
    let insert_time = Utc::now();
    if user_package_rows.len() > 0 {
        // update the package
        if rows.len() > 0 {
            // update table with new details
            let new_package_upload = &db
                .query(
                "UPDATE packages SET updatedAt = $1, description = $2, repository = $3, unlisted = $4 WHERE name = $2",
                &[&insert_time, &package.description, &package.repository, &package.unlisted, &package.name])
                .await?;
            Ok(NewPackageResult {
                ok: true,
                msg: "Success".to_owned(),
            })
        } else {
            Ok(NewPackageResult {
                ok: false,
                msg: "Not Found".to_owned(),
            })
        }
    } else {
        // check for exiting package
        if rows.len() > 0 {
            Ok(NewPackageResult {
                ok: false,
                msg: "Not Authorized".to_owned(),
            })
        } else {
            // creates a new package entry for the author
            let normalized_name = normalize(&package.name);
            let insert_time = Utc::now();
            let new_package_upload = &db
                .query(
                    "INSERT INTO packages (name, normalizedName, owner, description, repository, packageUploadNames, locked, malicious, unlisted, createdAt, updatedAt) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                    &[&package.name, &normalized_name, &user_package_rows.first().unwrap().get::<usize, String>(0), &package.description, &package.repository, &Array::<String>::from_vec(vec![], 0), &package.locked, &package.malicious, &package.unlisted, &insert_time, &insert_time]
                )
                .await?;
            // update user and push the new package name
            let mut package_names: Vec<String> = user_package_rows.first().unwrap().get::<usize, Array<String>>(4).iter().cloned().collect();
            package_names.push(package.name);
            let new_package_upload = &db
                .query(
                "UPDATE users SET packageNames = $1 WHERE name = $2",
                &[&Array::<String>::from_vec(package_names.clone(), package_names.len() as i32), &user_package_rows.first().unwrap().get::<usize, String>(0)])
                .await?;
            Ok(NewPackageResult {
                ok: true,
                msg: "Success".to_owned(),
            })
        }
    }
}


#[derive(Debug, Deserialize, Serialize, FromSql)]
pub struct Files {
    pub in_manifest: String,
    pub tx_id: String,
}


// TODO: implement upload creation
pub async fn create_package_uploads(
    db: Arc<Client>,
    package: NewPackageUpload,
    files: Files,
    prefix: String,
) -> Result<NewPackageResult, Error> {
    if !&package.upload {
        Ok(NewPackageResult {
            ok: true,
            msg: "Success".to_owned(),
        })
    } else {
        let rows = &db
            .query("SELECT * FROM packages WHERE name = $1", &[&package.name])
            .await?;
        if rows.len() > 0 {
            // TODO: insert new package into DB
            let new_package_name = format!("{}@{}", &package.name, &package.version);
            let insert_time = Utc::now();
            let new_package_upload = &db
             .query(
                  "INSERT INTO 'package-uploads' (name, package, entry, version, prefix, files, createdAt) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                  &[&new_package_name, &package.name, &package.entry, &package.version, &prefix, &Json::<Files>(files), &insert_time]
              )
             .await?;
            Ok(NewPackageResult {
                ok: true,
                msg: "Success".to_owned(),
            })
        } else {
            Ok(NewPackageResult {
                ok: false,
                msg: "Not found".to_owned(),
            })
        }
    }
}
