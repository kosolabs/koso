#!/usr/bin/env zsh

set -eo pipefail

if [ -z "${KOSO_IMAGE_DIGEST}" ]; then
    echo "KOSO_IMAGE_DIGEST variable must be set."
    exit 1
fi
if [ -z "${GITHUB_SHA}" ]; then
    echo "GITHUB_SHA variable must be set."
    exit 1
fi
if [ -z "${GHCR_USER}" ]; then
    echo "GHCR_USER variable must be set."
    exit 1
fi
if [ -z "${GHCR_TOKEN}" ]; then
    echo "GHCR_TOKEN variable must be set."
    exit 1
fi

echo "Deploying commit ${GITHUB_SHA}, image digest ${KOSO_IMAGE_DIGEST}"
cd /root/koso

# Checkout the repo at the given commit.
git status
git reset --hard
git clean -f -d
git fetch
git checkout $GITHUB_SHA
git status

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

# Bind a failure handler that will trigger rollbacks if things go wrong.
mkdir -p /root/rollouts
touch /root/rollouts/koso_deployed_sha /root/rollouts/koso_rollback_sha /root/rollouts/koso_deployed_image /root/rollouts/koso_rollback_image
ROLLBACK_SHA=$(cat /root/rollouts/koso_deployed_sha|tr -d '\n')
ROLLBACK_IMAGE=$(cat /root/rollouts/koso_deployed_image|tr -d '\n')
echo "If something goes wrong, will rollback to  commit ${ROLLBACK_SHA}, image digest ${ROLLBACK_IMAGE}"
if [ "${DISABLE_ROLLBACK}" != "true" ]; then
    function _on_fail {
        echo "Deploy failed, trying to rollback."
        if [ -z "${ROLLBACK_SHA}" ]; then
            echo "No rollback sha (/root/rollouts/koso_deployed_sha) present. Will not rollback"
        elif [ -z "${ROLLBACK_IMAGE}" ]; then
            echo "No rollback image (/root/rollouts/koso_deployed_image) present. Will not rollback"
        elif [[ "${ROLLBACK_SHA}" == "${GITHUB_SHA}" ]] && [[ "${ROLLBACK_IMAGE}" == "${KOSO_IMAGE_DIGEST}" ]]; then
            echo "Rollback target is the same as deployed. Will not rollback"
        else
            echo "Rolling back to commit ${ROLLBACK_SHA} and image ${ROLLBACK_IMAGE}"

            # First checkout the rollback target in case this script was changed.
            git reset --hard
            git clean -f -d
            git fetch
            git checkout $ROLLBACK_SHA
            git status

            systemctl status koso.service

            echo "Running ./deploy.sh in rollback mode..."
            DISABLE_ROLLBACK="true" GITHUB_SHA="${ROLLBACK_SHA}" KOSO_IMAGE_DIGEST="${ROLLBACK_IMAGE}" ./deploy.sh
            echo "Rollback complete."
        fi
        exit 1
    }
    trap _on_fail ZERR
fi

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
systemctl status koso.service
echo "Restarted service."

# Wait for the server to respond healthy.
echo "Health checking service..."
curl -sS --verbose --fail \
    --retry 30 \
    --retry-connrefused \
    --retry-delay 1 \
    http://localhost:3000/healthz
echo "Health check passed."

# Finally, after things are healthy, write the deployed state.
cp /root/rollouts/koso_deployed_sha /root/rollouts/koso_rollback_sha
cp /root/rollouts/koso_deployed_image /root/rollouts/koso_rollback_image
echo -n "${GITHUB_SHA}" > /root/rollouts/koso_deployed_sha
echo -n "${KOSO_IMAGE_DIGEST}" > /root/rollouts/koso_deployed_image

echo "Deployment complete"
