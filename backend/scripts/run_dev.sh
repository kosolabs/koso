#!/usr/bin/env zsh

set -eo pipefail

export DATABASE_URL=${DATABASE_URL:=postgresql://localhost/koso}
sqlx migrate run
TESTONLY_ENABLE_DEV=true GH_APP_ENV=dev cargo run