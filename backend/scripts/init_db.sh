#!/usr/bin/env bash
#set -x
#set -eo pipefail
if [ -z "${DATABASE_URL}" ]; then
    echo "DATABASE_URL variable must be set."
    exit 1
fi

echo "Initializing database $DATABASE_URL"

# Keep pinging Postgres until it's ready to accept commands
until psql ${DATABASE_URL} -c '\q'; do
  echo >&2 "Postgres is still unavailable - sleeping"
  sleep 1
done

if [[ -z "${KOSO_IMAGE}" ]]; then
  echo "KOSO_IMAGE not set. Running sqlx directly."
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
