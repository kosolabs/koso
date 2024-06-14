# create-svelte

Everything you need to build a Svelte project, powered by [`create-svelte`](https://github.com/sveltejs/kit/tree/main/packages/create-svelte).

## Creating a project

If you're seeing this, you've probably already done this step. Congrats!

```bash
# create a new project in the current directory
npm create svelte@latest

# create a new project in my-app
npm create svelte@latest my-app
```

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```bash
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```bash
npm run build
```

You can preview the production build with `npm run preview`.

> To deploy your app, you may need to install an [adapter](https://kit.svelte.dev/docs/adapters) for your target environment.

# Backend

## PostgresSQL

The backend uses a Postgres databse. There are several ways you can setup Postgress. [postgresapp](https://postgresapp.com/) is the simplest, but you may use one of the [alternatives](https://www.postgresql.org/download/macosx/) as you wish. The backend defaults to this connection string: `postgresql://localhost`. You may override this by setting the `DATABASE_URL` environment variable.

The backend uses SQLx to interact with Postgres. Install the [SQLx CLI](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) in order to run migrations and perform other administrative operations:

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

### Migrations

Add a migration:

```bash
sqlx migrate add some-meaningful-name
```

Run migrations

```bash
sqlx migrate run
```

## Developing

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

### Auto-reload

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
