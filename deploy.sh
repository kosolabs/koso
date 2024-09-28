#!/usr/bin/env zsh

set -e

function _on_fail {
    telegram "Failed to deploy $(git rev-parse --short HEAD) \\- $(git log --format=%s -n 1 HEAD | telegram_escape)" "❌"
}
trap _on_fail ZERR

source /root/.environment
source /root/.telegram.zsh

docker pull ghcr.io/kosolabs/koso:main

# TODO: Use an image tagged with the git revision being deployed. In koso.service too.
docker run \
    --add-host host.docker.internal:host-gateway \
    --env DATABASE_URL=postgresql://koso:koso@host.docker.internal/koso \
    --rm \
    ghcr.io/kosolabs/koso:main \
    "./sqlx" migrate run

systemctl daemon-reload
systemctl restart koso

telegram "$(git log --format='Deployed %h by %an - %s' -n 1 HEAD | telegram_escape)" "✅"
