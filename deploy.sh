#!/usr/bin/env zsh

set -e

function _on_fail {
    telegram "Failed to deploy $(git rev-parse --short HEAD) \\- $(git log --format=%s -n 1 HEAD | telegram_escape)" "❌"
}
trap _on_fail ZERR

source /root/.environment
source /root/.telegram.zsh

if [ -z "${KOSO_IMAGE_DIGEST}" ]; then
    echo "KOSO_IMAGE_DIGEST variable must be set."
    exit 1
fi

# Cleanup old images and containers
echo "Cleaning up stale images and containers..."
docker image prune -a --force --filter "until=32h"
docker container prune --force --filter "until=32h"
echo "Cleaned up stale images and containers."

echo "Deploying image ghcr.io/kosolabs/koso:$KOSO_IMAGE_DIGEST"

# Pull the new image
docker pull ghcr.io/kosolabs/koso:$KOSO_IMAGE_DIGEST

# Run DB migrations.
echo "Running database migrations..."
docker run \
    --add-host host.docker.internal:host-gateway \
    --env DATABASE_URL=postgresql://koso:koso@host.docker.internal/koso \
    --rm \
    ghcr.io/kosolabs/koso:$KOSO_IMAGE_DIGEST \
    "./sqlx" migrate run
echo "Finished database migrations."

# Set the image label in the systemctl override file.
cat >/etc/systemd/system/koso.service.d/override.conf <<EOL
[Service]
Environment="KOSO_IMAGE_LABEL=$KOSO_IMAGE_DIGEST"
EOL

# Load the updated koso.service file and restart on the new version.
echo "Restarting service..."
systemctl daemon-reload
systemctl restart koso@"$KOSO_IMAGE_DIGEST".service
echo "Restarted service."

telegram "$(git log --format='Deployed %h by %an - %s' -n 1 HEAD | telegram_escape)" "✅"
