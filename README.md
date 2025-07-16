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

1. Install the [Stripe CLI](https://docs.stripe.com/stripe-cli).

   ```sh
   brew install stripe/stripe-cli/stripe
   stripe login
   ```

1. Configure secrets.

   In the `koso` root folder, run:

   ```sh
   mkdir -p .secrets/koso
   openssl rand -hex 256 > .secrets/koso/hmac
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
   pnpm dev
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
- [httpBook - Rest Client](https://marketplace.visualstudio.com/items?itemName=anweber.httpbook)

### Backend Interactions

Once a server has been started, you can interact with it at http://localhost:3000. There are example requests in [koso.http](backend/koso.http) which you can run with [httpBook](https://marketplace.visualstudio.com/items?itemName=anweber.httpbook).

### Running a Built Frontend with the Backend

This setup is similar to how the app will run in production. A single server serves the API, WebSocket, and static frontend files.

1. In the `frontend` folder, run:

   ```bash
   pnpm build
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

### One-time setup

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

# Integrations

## Cloudflare Reverse Proxy

### One-time Setup

Local development of integrations requires that external services be able to talk to your locally running server. This is accomplished via a reverse proxy.

1. Install [cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/).

   ```sh
   brew install cloudflared
   ```

1. Login to Cloudflare.

   ```sh
   cloudflared login
   ```

1. Authorize Cloudflare Tunnel to the **koso.app** domain.

1. Click **Authorize**.

1. Create a tunnel.

   ```sh
   cloudflared tunnel create koso
   ```

1. Copy the **tunnel ID** to the clipboard.

1. Configure the DNS for the tunnel. Replace `namespace` with an appropriate name, e.g. shad-dev.

   ```sh
   cloudflared tunnel route dns koso ${namespace}.koso.app
   ```

1. Write the following file to `~/.cloudflared/config.yaml`. Replace `${tunnel-id}` with the value from the output of the tunnel create command.

   ```yaml
   url: http://localhost:3000
   tunnel: ${tunnel-id}
   credentials-file: ~/.cloudflared/${tunnel-id}.json
   ```

### Testing locally

1. Start the reverse proxy server.

   ```sh
   cloudflared tunnel run koso
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

## Discord

### One-time setup

1. Variables used in this section:

   - `${title}`, e.g. `ShadDev`
   - `${namespace}`, e.g. `shad-dev`

1. Navigate to https://discord.com/developers/applications.
1. Click **New Application**.
1. Set the name of the app to `${title}Koso`, e.g. `ShadDevKoso`.
1. Copy the Application ID and save it for later (`${application-id}`).
1. Copy the public key into `.secrets/discord/public_key`.
1. Set the Interactions Endpoint URL to: `https://${namespace}.koso.app/api/notifiers/discord/interaction`
1. Click **Save Changes**.
1. Click **Bot**.
1. Click **Reset Token**.
1. Click **Yes, do it!**.
1. Enter your password and click **Submit** if necessary.
1. Copy the **Token** into `.secrets/discord/token`.
1. Install the `/token` command into your discord app:

   ```sh
   curl -X POST "https://discord.com/api/v10/applications/${application-id}/commands" \
     -H "Authorization: Bot $(cat .secrets/discord/token)" \
     -H "Content-Type: application/json" \
     -d '{
       "name": "token",
       "description": "Start the authorization flow for Koso"
     }'
   ```

1. Replace `${application-id}` in the link below with the value from above, then follow the link to install the app.

   https://discord.com/oauth2/authorize?client_id=${application-id}&permissions=2048&integration_type=0&scope=bot

1. Start a conversation with the bot (`@${title}Koso`).
1. Click the **Commands** button.
1. Click the **Send** button corresponding to the `/token` command.
1. Follow the link to authorize your local Koso to send notifications via Discord.

## Slack

### One-time setup

1. Variables used in this section:

   - `${title}`, e.g. `ShadDev`
   - `${namespace}`, e.g. `shad-dev`

1. Navigate to https://api.slack.com/apps.
1. Click **Create New App**.
1. Click **From a manifest**.
1. Set your workspace to: Koso Labs.
1. Click **Next**.
1. Copy / paste the following manifest replacing the variables as necessary:

   ```json
   {
     "display_information": {
       "name": "${title}Koso",
       "description": "Receive notification updates from ${title}Koso"
     },
     "features": {
       "bot_user": {
         "display_name": "${title}Kosobot",
         "always_online": true
       },
       "slash_commands": [
         {
           "command": "/${namespace}-token",
           "url": "https://${namespace}.koso.app/api/notifiers/slack/command",
           "description": "Start the authorization flow",
           "should_escape": false
         }
       ]
     },
     "oauth_config": {
       "scopes": {
         "bot": ["chat:write", "im:history", "im:read", "im:write", "commands"]
       }
     },
     "settings": {
       "interactivity": {
         "is_enabled": true,
         "request_url": "https://${namespace}.koso.app/api/notifiers/slack/interact"
       },
       "org_deploy_enabled": false,
       "socket_mode_enabled": false,
       "token_rotation_enabled": false
     }
   }
   ```

1. Copy the **Signing Secret** into `.secrets/slack/signing_secret`.
1. Click **Install App**.
1. Click **Install to Koso Labs**.
1. Click **Allow**.
1. Copy the **Bot User OAuth Token** into `.secrets/slack/token`.
1. Click **App Home**.
1. In the **Show Tabs** section, check the box **Allow users to send Slash commands and messages from the messages tab**.
1. In Slack, click your app in the Apps section.
1. Click on **About**.
1. Next to the **token** command, click **Start command**.
1. Click the **Send** button.
1. Click **Authorize Koso** to authorize your local Koso to send notifications via Slack.

### Testing locally

You can send a test message from the [profile page](http://localhost:5173/profile).

## Telegram

### One-time setup

Create a new bot and configure the secrets.

1. Variables used in this section:

   - `${title}`, e.g. `ShadDev`
   - `${namespace}`, e.g. `shad-dev`

1. Send a Telegram message to @BotFather: `/newbot`.

1. Name the bot `${title}Kosobot`.

1. Copy the access token into `.secrets/telegram/token`.

1. Generate a secret token that will authorize Telegram to call the webhook and save it into `.secrets/telegram/secret_token`.

   ```sh
   openssl rand -base64 256 | tr -dc 'A-Za-z0-9' | head -c 256 > .secrets/telegram/secret_token
   ```

1. Configure your Telegram bot to use the secret_token with your webhook.

   ```sh
   curl -X POST "https://api.telegram.org/bot$(cat .secrets/telegram/token)/setWebhook" \
     -d "url=https://${namespace}.koso.app/api/notifiers/telegram/webhook" \
     -d "secret_token=$(cat .secrets/telegram/secret_token)"
   ```

1. Send a `/token` message to your bot: [${title}Kosobot](https://t.me/${title}Kosobot).

1. Follow the link to authorize your local Koso to send notifications via Telegram.

### Testing locally

With setup complete you can interact with tasks and generate notifications. Note though that
most self notifications are suppressed. Use [login_test_user.sh](backend/scripts/login_test_user.sh)
to login as a test user, interact with tasks and trigger notifications.

## MCP

[docs](https://modelcontextprotocol.io)

### One-time setup

#### Auth

1. Log in to your local server and open the developer console
1. Run `localStorage.credential`
1. Copy the credential for usage below.

#### VS Code

[docs](https://code.visualstudio.com/docs/copilot/chat/mcp-servers)

1. Open the command palette (`cmd+shift+p`) and run `MCP: Open User Configuration`
1. Insert the Koso server
   ```json
   {
     "servers": {
       "koso-mcp": {
         "url": "http://localhost:3000/api/mcp/sse",
         "type": "http"
       }
     },
     "inputs": []
   }
   ```
1. Open a Copilot chat, click on `Configure tools` and enable Koso

#### Claude Code

[Claud docs](https://docs.anthropic.com/en/docs/claude-code/mcp#add-mcp-servers-from-json-configuration)

1. Install Claude Code: `npm install -g @anthropic-ai/claude-code`
1. Run the setup flow: `claude`
1. Add Koso: `claude mcp add --transport http koso-mcp http://localhost:3000/api/mcp/sse`

#### MCP Inspector

Use the [MCP Inspector](https://github.com/modelcontextprotocol/inspector) to test the server without a model.

Setup:

1. Run the inspector: `npx @modelcontextprotocol/inspector`
1. Enter the server URL ()`http://localhost:3000/api/mcp/sse`) and click Connect
1. In the authentication section, paste the auth token from above in the Bearer Token field
