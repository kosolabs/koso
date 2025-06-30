## Development Environment (MacOS)

### First Time Setup

1. Install [Homebrew](https://brew.sh/).

   ```sh
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

1. Install and start [PostgreSQL](https://www.postgresql.org/).

   ```sh
   POSTGRESQL_VERSION=17
   brew install postgresql@$POSTGRESQL_VERSION
   brew services start postgresql@$POSTGRESQL_VERSION
   ```

1. Install [Rust](https://www.rust-lang.org/).

   ```sh
   brew install rustup
   rustup-init
   ```

1. The backend uses SQLx to interact with PostgreSQL. Install the [SQLx](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) in order to run migrations and perform other administrative operations.

   ```sh
   cargo install sqlx-cli --no-default-features --features rustls,postgres
   ```

1. Configure SQLx with the DATABASE_URL.

   ```sh
   export DATABASE_URL=postgresql://localhost/koso
   ```

   Also, add the environment variable to the appropriate profile file (`~/.profile`, `~/.bash_profile`, `~/.bashrc`, `~/.zshrc`, `~/.zshenv`) so you don't have to run it every time.

1. Create the database and run the DB migrations.

   In the `backend` folder, run:

   ```sh
   sqlx database create
   sqlx migrate run
   ```

1. Install [Node.js](https://nodejs.org/).

   ```sh
   brew install node pnpm
   ```

1. Install the [Stripe CLI](https://docs.stripe.com/stripe-cli)

   ```sh
   brew install stripe/stripe-cli/stripe
   stripe login
   ```

### Once A Day / After Every Pull

1. Run the most recent DB migrations.

   In the `backend` folder, run:

   ```sh
   sqlx migrate run
   ```

1. Install the latest frontend dependencies.

   In the `frontend` folder, run:

   ```sh
   pnpm install
   ```

### Start Backend and Frontend

1. Start the backend server.

   In the `backend` folder, run:

   ```sh
   cargo run
   ```

1. Start the frontend server.

   In the `frontend` folder, run:

   ```sh
   pnpm run dev
   ```

1. Navigate to http://localhost:5173/

### VS Code

The [Koso Workspace](koso.code-workspace) is configured for development in VS Code.

The following plugins are recommended:

- [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)
- [Svelte for VS Code](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)
- [Tailwind CSS IntelliSense](https://marketplace.visualstudio.com/items?itemName=bradlc.vscode-tailwindcss)
- [Prettier - Code Formatter](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)
- [ESLint](https://marketplace.visualstudio.com/items?itemName=dbaeumer.vscode-eslint)
- [Vitest](https://marketplace.visualstudio.com/items?itemName=vitest.explorer)
- [Playwright Test for VSCode](https://marketplace.visualstudio.com/items?itemName=ms-playwright.playwright)

### Backend Interactions

Once a server has been started, you can interact with it at http://localhost:3000. There are example requests in [koso.http](backend/koso.http) which you can run with [REST Client](https://marketplace.visualstudio.com/items?itemName=humao.rest-client).

### Backend Auto-reload

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

### Running a Built Frontend with the Backend

This setup is similar to how the app will run in production. A single server serves the API, WebSocket, and static frontend files.

1. In the `frontend` folder, run:

   ```bash
   pnpm run build
   ```

1. In the `backend` folder, run the server:

   ```bash
   cargo run
   ```

This will create a `frontend/build` folder. The `backend/static` folder is symlinked to that folder and will serve the compiled frontend directly from the backend.

### Running playwright tests

Playwright tests, i.e. integration tests, flex the entire system end-to-end via the frontend testing
framework Playwright. The tests run as part of CI, but you may also run them locally.

#### Option A: Iterate quickly by running against your development server

Make changes and run the tests quickly without rebuilding the world. Start a frontend and backend
server in the usual manner, see above, and run the tests in VSCode using the Playwright extension or
via the CLI:

```bash
pnpm exec playwright test
```

#### Option B: Mimic production and run against a built frontend with production

Follow "Running a Built Frontend with the Backend" above to build the frontend and run the backend.
Run the tests:

```bash
PW_SERVER_PORT=3000 pnpm exec playwright test
```

#### Option C: Mimic CI and build things from scratch

This is what our CI workflows do. playwright will build the frontend and run a backend for the
duration of the tests:

```bash
CI=true pnpm exec playwright test
```

## Development Docker builds

Build and run the docker image defined in `Dockerfile`.

### One time setup

1. Download and install Docker: https://www.docker.com/products/docker-desktop/

### Build & run

1. Build the image:

   ```bash
   DOCKER_DEFAULT_PLATFORM=linux/amd64 docker build -t ghcr.io/kosolabs/koso .
   ```

1. Configure the DATABASE_URL.

   ```sh
   export DATABASE_URL=postgresql://localhost/koso
   ```

   Also, add the environment variable to the appropriate profile file (`~/.profile`, `~/.bash_profile`, `~/.bashrc`, `~/.zshrc`, `~/.zshenv`) so you don't have to run it every time.

1. Run database migrations:

   ```bash
   DOCKER_DEFAULT_PLATFORM=linux/amd64 docker run \
      --env DATABASE_URL \
      --network=host \
      --rm -it \
      ghcr.io/kosolabs/koso:latest \
      "./sqlx" migrate run
   ```

1. Run the server:

   ```bash
   DOCKER_DEFAULT_PLATFORM=linux/amd64 docker run \
      --env KOSO_ENV=dev \
      -v $HOME/.secrets:/.secrets \
      --network=host \
      --rm -it \
      ghcr.io/kosolabs/koso:latest
   ```

## Server setup

Commands for setting up a new server are in [setup_server.sh](backend/scripts/setup_server.sh)

### Environment

We use a [Github Environment](https://docs.github.com/en/actions/managing-workflow-runs-and-deployments/managing-deployments/managing-environments-for-deployment) configured on the `Deploy` workflow which exposes a `KOSO_KEY` to access the server.

## Postgres

### Migrations

Add a migration:

```bash
sqlx migrate add some-meaningful-name
```

Run migrations

```bash
sqlx migrate run
```

### Backups

[psql_backup.sh](backend/scripts/psql_backup.sh) exports backups of our Postgresql DB to cloud storage.

The script is ran by a daily cron and logs are available at `koso-psql-backups/backups.log`.

Backups are stored in a GCP cloud storage bucket named `koso-psql-backups`. The bucket has
soft deletion and object versioning configured, along with lifecycle rules to
auto-delete objects after 30 days.

#### Restore

Identify the backup to restore in the cloud console and update `backup_name` below with the target object name.

```bash
backup_name=TARGET-backup.sql.gz
```

Download and unzip the backup:

```bash
backup_object=gs://$koso-psql-backups/$backup_name
gcloud storage cp --print-created-message $backup_object ./
gzip -dk $backup_name
```

Restore the backup:

```bash
PGPASSWORD=$PSQL_PASSWORD pg_restore \
   --host="$PSQL_HOST" \
   --port="$PSQL_PORT" \
   --db="$PSQL_DB" \
   --username="$PSQL_USER" \
   -f \
   $backup_name
```

### Upgrade

Upgrade Postgres to a new major version. In the example below, from 16 to 17.

1. Update the postgres image version from postgres:16 to postgres:17 in ci.yml and merge.

1. Install the new version of posgres:

   ```bash
   sudo apt update
   sudo apt install postgresql-17
   pg_lsclusters
   ```

1. Backup the cluster just in case:

   ```bash
   pg_dumpall > ~/postgres-dump-$(date -u "+%Y-%m-%dT%H-%M-%S-%3NZ")
   ```

1. Upgrade the cluster:

   ```
   sudo service postgresql stop
   sudo pg_renamecluster 17 main main_pristine
   sudo pg_upgradecluster 16 main
   sudo service postgresql start

   pg_lsclusters
   ```

1. Verify the new cluster is working by visiting our app and verifying things work. Look at backend logs as well for anything suspicious.

   ```bash
   pg_lsclusters
   ```

1. Drop the old and transition version:

   ```bash
   sudo pg_dropcluster 16 main --stop
   sudo pg_dropcluster 17 main_pristine --stop
   ```

## Github Webhooks

References:

- https://docs.github.com/en/webhooks/webhook-events-and-payloads
- https://docs.github.com/en/apps/creating-github-apps/registering-a-github-app/using-webhooks-with-github-apps
- https://docs.github.com/en/webhooks/testing-and-troubleshooting-webhooks/testing-webhooks

### One-time setup

Install [Smee](https://smee.io/)

```bash
npm install --global smee-client
```

### Testing locally

After starting your local server:

1. Configure your development webhook secret in: `.secrets/github/webhook_secret`
1. Start a new Smee channel: https://smee.io/
1. Start smee locally with the new channel `smee -u $CHANNEL_URL --port 3000 --path /plugins/github/app/webhook`
1. Trigger or [redeliver](https://docs.github.com/en/webhooks/testing-and-troubleshooting-webhooks/redelivering-webhooks#redelivering-github-app-webhooks) some events

## Stripe

### One-time setup

1. Install the [CLI](https://docs.stripe.com/stripe-cli#install):

   ```bash
   brew install stripe/stripe-cli/stripe
   stripe login
   ```

1. Configure your sandbox secret API key in `.secrets/stripe/secret_key`

1. Configure your sandbox webhook secret

   ```bash
   stripe listen --api-key $(cat .secrets/stripe/secret_key) --print-secret > .secrets/stripe/webhook_secret
   ```

### Testing locally

We use the **Koso Labs Sandbox** Stripe sandbox for testing. Login to Stripe and switch to the Sandbox to find API keys and webhook details. Feel free to create a new sandbox if needed.

Start a local listener with [stripe listen](https://docs.stripe.com/cli/listen):

```bash
./backend/scripts/stripe_listen.sh
```

With this in place and your local servers running, you can:

- Run playwright subscription tests
- Trigger events on demand with `stripe trigger`. For example: `stripe trigger checkout.session.completed`
- Test interactively using the `4242 4242 4242 4242` card number: https://docs.stripe.com/testing#testing-interactively

## Telegram

### One time setup

Create a new bot and configure the secrets.

1. Send a Telegram message to @BotFather: `/newbot`
1. Name the bot, e.g. UserDevBot
1. Copy the access token into `.secrets/telegram/token`
1. Generate an HMAC key for signing tokens: `openssl rand -base64 256 > .secrets/koso/hmac`
1. Restart your servers and authorize Telegram on your profile page

### Testing locally

With setup complete you can interact with tasks and generate notifications. Note though that
most self notifications are suppressed. Use [login_test_user.sh](backend/scripts/login_test_user.sh)
to login as a test user, interact with tasks and trigger notifications.
