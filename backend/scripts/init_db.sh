#!/usr/bin/env bash
#set -x
#set -eo pipefail
DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=postgres}"
DB_PORT="${POSTGRES_PORT:=5432}"

# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  echo >&2 "Postgres is still unavailable - sleeping"
  sleep 1
done

echo >&2 "Postgres is up and running on port ${DB_PORT}!"
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}

if [[ -z "${KOSO_IMAGE}" ]]; then
  sqlx database create
  sqlx migrate run
else
  docker run \
    --env DATABASE_URL \
    --network=host \
    --rm \
    $KOSO_IMAGE \
    "./sqlx" database create
  docker run \
    --env DATABASE_URL \
    --network=host \
    --rm \
    $KOSO_IMAGE \
    "./sqlx" migrate run
fi
echo >&2 "Postgres has been migrated, ready to go!"
