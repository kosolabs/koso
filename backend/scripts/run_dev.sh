#!/usr/bin/env zsh

set -eo pipefail

export DATABASE_URL=${DATABASE_URL:=postgresql://localhost/koso}
sqlx migrate run
cargo run
