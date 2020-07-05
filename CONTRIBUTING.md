# Contributing

Thank you for taking the time to contribute! ♥️

## PostgreSQL

If you don't already have Postgres installed on your machine, do that now.

If you are on macOS, run the following commands:

```sh
$ brew install postgresql
$ brew services start postgresql
```

Congrats! You have now installed Postgres. Now it's time to set it up and configure it.

Enter the PostgresQL console:

```sh
$ psql postgres
```

Create the DB, create a new user `nest`, and give that user access to the new database:

```sql
CREATE DATABASE nest;

CREATE USER nest WITH ENCRYPTED PASSWORD '123';

GRANT ALL PRIVILEGES ON DATABASE nest TO nest;
```

Awesome, now log into that user with the following command:

```sh
$ psql -U nest -W -d nest -h localhost
```

Enter `123` when prompted for the password.

Now create some data tables:

```sql
CREATE TABLE users (
  name VARCHAR(20) NOT NULL UNIQUE,
  normalizedName VARCHAR(20) NOT NULL UNIQUE,
  password VARCHAR(256) NOT NULL,
  apiKey VARCHAR(256) NOT NULL,
  packageNames VARCHAR [],
  createdAt timestamptz
);

CREATE TABLE packages (
  name VARCHAR(40) NOT NULL UNIQUE,
  normalizedName VARCHAR(40) NOT NULL UNIQUE,
  owner VARCHAR(250) NOT NULL,
  description TEXT,
  repository TEXT,
  latestVersion VARCHAR(61),
  latestStableVersion VARCHAR(61),
  packageUploadNames VARCHAR [],
  locked BOOLEAN NOT NULL,
  malicious BOOLEAN NOT NULL,
  unlisted  BOOLEAN NOT NULL,
  updatedAt timestamptz,
  createdAt timestamptz
);

CREATE TABLE "package-uploads" (
  name VARCHAR(40) NOT NULL UNIQUE,
  package VARCHAR(40) NOT NULL,
  entry VARCHAR(60),
  version VARCHAR(20) NOT NULL,
  prefix VARCHAR(20),
  malicious BOOLEAN,
  files JSON,
  createdAt timestamptz
);
```

Add some dummy data to your DB:

```sql
INSERT INTO users (name, normalizedName, password, apiKey, packageNames, createdAt) VALUES ('divy', 'divy', 'weird-password@ok-boomer', 'haha', ARRAY [ 'sass' ], '2016-06-22 19:10:25-07');

INSERT INTO packages (name, normalizedName, owner, description, repository, latestVersion, latestStableVersion, packageUploadNames, locked, malicious, unlisted, createdAt, updatedAt) VALUES ('sass', 'sass', 'divy', 'Deno Sass Compiler', 'https://github.com/divy-work/deno-sass', 'v0.2.0', 'v0.2.0', ARRAY ['sass'], false, false, false, '2016-06-22 19:10:25-07', '2016-06-22 19:10:25-07');
```

## Rust

If you don't already have Rust installed on your machine, do that now.

```sh
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Brilliant. Now you can use Rust.

## Running the API

```sh
$ cargo run # or ``cargo watch -x run``
```

Now visit `http://127.0.0.1:8080/graphiql`
