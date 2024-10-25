#!/usr/bin/env zsh

set -e

if [ -z "${KOSO_IMAGE_DIGEST}" ]; then
    echo "KOSO_IMAGE_DIGEST variable must be set."
    exit 1
fi

# Cleanup old images and containers
echo "Cleaning up stale images and containers..."
docker image prune -a --force --filter "until=32h"
docker container prune --force --filter "until=32h"
echo "Cleaned up stale images and containers."

echo "Deploying image ghcr.io/kosolabs/koso@$KOSO_IMAGE_DIGEST"

# Pull the new image
echo $GHCR_TOKEN | docker login ghcr.io -u $GHCR_USER --password-stdin
docker pull ghcr.io/kosolabs/koso@$KOSO_IMAGE_DIGEST

# Run DB migrations.
echo "Running database migrations..."
docker run \
    --env DATABASE_URL=postgresql://koso:koso@localhost/koso \
    --network=host \
    --rm \
    ghcr.io/kosolabs/koso@$KOSO_IMAGE_DIGEST \
    "./sqlx" migrate run
echo "Finished database migrations."

# Setup backups
mkdir -p /root/koso-psql-backups
cp -r /root/koso/backend/psql_backups/. /root/koso-psql-backups/
crontab -u root /root/koso-psql-backups/psql_backup_cron.txt

# Copy over the latest systemctl unit file.
cp backend/koso.service /etc/systemd/system/koso.service

# Set the image label in the systemctl override file.
mkdir -p /etc/systemd/system/koso.service.d/
cat >/etc/systemd/system/koso.service.d/override.conf <<EOL
[Service]
Environment="KOSO_IMAGE_DIGEST=$KOSO_IMAGE_DIGEST"
EOL

# Load the updated koso.service file and restart on the new version.
echo "Restarting service..."
systemctl daemon-reload
systemctl restart koso.service
systemctl is-active koso.service && echo Koso service is running
systemctl enable koso.service
echo "Restarted service."

echo "Deployment complete"