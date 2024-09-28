#!/usr/bin/env zsh

set -e

function _on_fail {
    telegram "Failed to deploy $(git rev-parse --short HEAD) \\- $(git log --format=%s -n 1 HEAD | telegram_escape)" "❌"
}
trap _on_fail ZERR

source /root/.environment
source /root/.telegram.zsh

# Cleanup old images and containers
docker image prune -a --force --filter "until=32h"
docker container prune --force --filter "until=32h"

# Pull the new image
docker pull ghcr.io/kosolabs/koso:main

# Run DB migrations.
docker run \
    --add-host host.docker.internal:host-gateway \
    --env DATABASE_URL=postgresql://koso:koso@host.docker.internal/koso \
    --rm \
    ghcr.io/kosolabs/koso:main \
    "./sqlx" migrate run

# Load the updated koso.service file and restart on the new version.
systemctl daemon-reload
systemctl restart koso

telegram "$(git log --format='Deployed %h by %an - %s' -n 1 HEAD | telegram_escape)" "✅"
