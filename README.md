## Frontend

The frontend uses [Svelte](https://svelte.dev/)

### Developing

First time only, install dependencies.

```bash
npm install
```

Start the dev server.

```bash
npm run dev
```

## Backend

### PostgresSQL

The backend uses a Postgres databse. There are several ways you can setup Postgress. [postgresapp](https://postgresapp.com/) is the simplest, but you may use one of the [alternatives](https://www.postgresql.org/download/macosx/) as you wish. The backend defaults to this connection string: `postgresql://localhost`. You may override this by setting the `DATABASE_URL` environment variable.

The backend uses SQLx to interact with Postgres. Install the [SQLx CLI](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) in order to run migrations and perform other administrative operations:

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

#### Migrations

Add a migration:

```bash
sqlx migrate add some-meaningful-name
```

Run migrations

```bash
sqlx migrate run
```

### Developing

Start a development server:

```bash
cargo run
```

You can interact with the server at http://localhost:3000 using a browser, a CLI tool like cURL or an API explorer such as [Postman](https://www.postman.com/downloads/).

List tasks:

```bash
curl localhost:3000/task/list | jq
```

Create a task:

```bash
curl -H 'Content-Type: application/json' \
  -d "{\"name\": \"$USER task $(date '+%d/%m/%Y_%H:%M:%S')\"}" \
  -X POST \
  localhost:3000/task/create | jq
```

#### Auto-reload

Tired of manually restarting your server after editing the code? Use systemfd and cargo-watch to
automatically recompile and restart the server whenever the source code changes.
It uses listenfd to be able to migrate connections from an old version of the app to a newly-compiled version.

One time setup:

```bash
cargo install cargo-watch systemfd
```

Running:

```bash
systemfd --no-pid -s http::3000 -- cargo watch -x run
```
