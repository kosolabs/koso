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
   brew install node
   ```

1. Install the frontend dependencies.

   In the `frontend` folder, run:

   ```sh
   npm install
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
   npm run dev
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
   npm run build
   ```

1. In the `backend` folder, run the server:

   ```bash
   systemfd --no-pid -s http::3000 -- cargo watch -x run
   ```

This will create a `frontend/build` folder. The `backend/static` folder is symlinked to that folder and will serve the compiled frontend directly from the backend.
