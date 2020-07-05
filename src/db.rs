// Postgres database management for Nest API

use crate::schema::{NewPackage, NewPackageResult, NewPackageUpload, NewUser, Package, User};
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
    let apiKey = create_api_key();
    let currTime = Utc::now();
    let normalizedName = normalize(&new_user.name);
    let rows = &db
        .query("INSERT INTO users (name, normalizedName, password, apiKey, packageNames, createdAt) VALUES ($1, $2, $3, $4, $5, $6)", &[&new_user.name, &normalizedName, &new_user.password, &apiKey, &Array::<String>::from_vec(vec![], 0), &currTime])
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
    let userPackageRows = &db
        .query(
            "SELECT * FROM users WHERE apiKey = $1 AND $2 = ANY(packageNames)",
            &[&package.apiKey, &package.name],
        )
        .await?;
    let rows = &db
        .query("SELECT * FROM packages WHERE name = $1", &[&package.name])
        .await?;
    let normalizedName = normalize(&package.name);
    let insertTime = Utc::now();
    if userPackageRows.len() > 0 {
        // update the package
        if rows.len() > 0 {
            // update table with new details
            let newPackageUpload = &db
                .query(
                "UPDATE packages SET updatedAt = $1, description = $2, repository = $3, unlisted = $4 WHERE name = $2",
                &[&insertTime, &package.description, &package.repository, &package.unlisted, &package.name])
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
            let normalizedName = normalize(&package.name);
            let insertTime = Utc::now();
            let newPackageUpload = &db
                .query(
                    "INSERT INTO packages (name, normalizedName, owner, description, repository, packageUploadNames, locked, malicious, unlisted, createdAt, updatedAt) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                    &[&package.name, &normalizedName, &userPackageRows.first().unwrap().get::<usize, String>(0), &package.description, &package.repository, &Array::<String>::from_vec(vec![], 0), &package.locked, &package.malicious, &package.unlisted, &insertTime, &insertTime]
                )
                .await?;
            // update user and push the new package name
            let mut packageNames: Vec<String> = userPackageRows.first().unwrap().get::<usize, Array<String>>(4).iter().cloned().collect();
            packageNames.push(package.name);
            let newPackageUpload = &db
                .query(
                "UPDATE users SET packageNames = $1 WHERE name = $2",
                &[&Array::<String>::from_vec(packageNames.clone(), packageNames.len() as i32), &userPackageRows.first().unwrap().get::<usize, String>(0)])
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
    pub inManifest: String,
    pub txId: String,
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
            let newPackageName = format!("{}@{}", &package.name, &package.version);
            let insertTime = Utc::now();
            let newPackageUpload = &db
             .query(
                  "INSERT INTO 'package-uploads' (name, package, entry, version, prefix, files, createdAt) VALUES ($1, $2, $3, $4, $5, $6, $7)",
                  &[&newPackageName, &package.name, &package.entry, &package.version, &prefix, &Json::<Files>(files), &insertTime]
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
