## Development Environment (MacOS)

### First Time Setup

1. Install [Homebrew](https://brew.sh/).

   ```sh
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

1. Install [PostgreSQL](https://www.postgresql.org/).

   ```sh
   brew install postgresql@16
   ```

1. Start PostgreSQL.

   ```sh
   brew services start postgresql@16
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

1. Install the frontend dependencies.

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
- [REST Client](https://marketplace.visualstudio.com/items?itemName=humao.rest-client)
- [Tailwind CSS IntelliSense](https://marketplace.visualstudio.com/items?itemName=bradlc.vscode-tailwindcss)
- [Prettier - Code Formatter](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)

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

Build the image:

```bash
DOCKER_DEFAULT_PLATFORM=linux/amd64 docker build -t ghcr.io/kosolabs/koso .
```

Run database migrations:

```bash
DOCKER_DEFAULT_PLATFORM=linux/amd64 docker run \
   --env DATABASE_URL=postgresql://$USER@localhost/$USER \
   --network=host \
   --rm -it \
   ghcr.io/kosolabs/koso:latest \
   "./sqlx" migrate run
```

Run the server:

```bash
DOCKER_DEFAULT_PLATFORM=linux/amd64 docker run \
   --env DATABASE_URL=postgresql://$USER@localhost/$USER \
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

1. Add 172.17.0.1 to /etc/postgresql/16/main/postgresql.conf:

   ```
   listen_addresses = 'localhost,172.17.0.1'
   ```

1. Add entry to /etc/postgresql/16/main/pg_hba.conf:

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
