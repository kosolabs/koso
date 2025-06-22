## Development Environment (MacOS)

### First Time Setup

1. Install [Homebrew](https://brew.sh/).

   ```sh
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

1. Install [PostgreSQL](https://www.postgresql.org/).

   ```sh
   brew install postgresql@17
   ```

1. Start PostgreSQL.

   ```sh
   brew services start postgresql@17
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

### DB Migrations

Add a migration:

```bash
sqlx migrate add some-meaningful-name
```

Run migrations

```bash
sqlx migrate run
```

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
   systemfd --no-pid -s http::3000 -- cargo watch -x run
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

1. Install docker:

   ```bash
   sudo su &&\
   apt update &&\
   apt install ca-certificates curl gnupg apt-transport-https gpg
   curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker.gpg
   apt update
   echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker.gpg] https://download.docker.com/linux/debian bookworm stable" |tee /etc/apt/sources.list.d/docker.list > /dev/null
   apt install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin docker-compose
   systemctl is-active docker

   echo $PULL_TOKEN| docker login ghcr.io -u $USER --password-stdin
   ```

### Environment

We use a [Github Environment](https://docs.github.com/en/actions/managing-workflow-runs-and-deployments/managing-deployments/managing-environments-for-deployment) configured on the `Deploy` workflow which exposes a `KOSO_KEY` to access the server.

### Access in bridge mode (old)

1. Add 172.17.0.1 to /etc/postgresql/17/main/postgresql.conf:

   ```
   listen_addresses = 'localhost,172.17.0.1'
   ```

1. Add entry to /etc/postgresql/17/main/pg_hba.conf:

   ```
   # Allow docker bridge
   host    all             all             172.0.0.0/8             scram-sha-256
   ```

### Configure github ssh keys (old)

https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys#set-up-deploy-keys

```bash
ssh-keygen -t ed25519 -C "koso-github-read-key" -f /root/.ssh/koso_github_read_id_ed25519 -N ''
eval "$(ssh-agent -s)"
cat >>/root/.ssh/config <<EOL
Host github.com
  AddKeysToAgent yes
  IdentityFile  ~/.ssh/koso_github_read_id_ed25519
EOL
# MANUAL - add a new deploy key with the public key (e.g. ssh-ed25519 KEY) to https://github.com/kosolabs/koso/settings/keys/new
cat /root/.ssh/koso_github_read_id_ed25519.pub
ssh -T git@github.com && echo "Github auth works"
```

### Server Github access (old)

Rather than using our personal key and since we only need read access, we use [Github Deploy Keys](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys#deploy-keys) to authenticate with Github from our server.

## Postgres

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

1. Configure your development webhook secret in: `koso/.secrets/github/webhook_secret`
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

### Testing locally

We use the **Koso Labs Sandbox** Stripe sandbox for testing. Login to Stripe and switch to the Sandbox to find API keys and webhook details. Feel free to create a new sandbox if needed.

After starting your local server:

1. Configure your sandbox secret API key in `koso/.secrets/stripe/secret_key`
1. Configure your sandbox webhook secret

   ```bash
   stripe listen --api-key $(cat koso/.secrets/stripe/secret_key) --print-secret > koso/.secrets/stripe/webhook_secret
   ```

1. Start a local listener with [stripe listen](https://docs.stripe.com/cli/listen). Add events as needed. Omit the API key to use an empemeral test environment.

   ```bash
   stripe listen \
      --forward-to localhost:3000/api/billing/stripe/webhook \
      --api-key=$(cat koso/.secrets/stripe/secret_key) \
      --events=checkout.session.completed,invoice.paid,customer.subscription.created,customer.subscription.deleted,customer.subscription.paused,customer.subscription.resumed,customer.subscription.updated
   ```

With this in place and your local servers running, you can:

- Trigger events on demand with `stripe trigger`. For example: `stripe trigger checkout.session.completed`
- Test interactively using the `4242 4242 4242 4242` card number: https://docs.stripe.com/testing#testing-interactively
