#!/usr/bin/env zsh

set -eo pipefail

DATABASE_URL=${DATABASE_URL:=postgresql://localhost/koso} sqlx migrate run
cargo run
