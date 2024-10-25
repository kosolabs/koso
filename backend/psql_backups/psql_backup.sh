#!/usr/bin/env zsh
set -eo pipefail

BACKUP_DIR=${BACKUP_DIR:-/root/koso-psql-backups}
BUCKET=${BUCKET:-koso-psql-backups}
PSQL_HOST=${PSQL_HOST:-localhost}
PSQL_PORT=${PSQL_PORT:-5432}
PSQL_DB=${PSQL_DB:-koso}
PSQL_USER=${PSQL_USER:-koso}
PSQL_PASSWORD=${PSQL_PASSWORD:-koso}

if [ ! -d "$BACKUP_DIR" ]; then
    echo "Directory $BACKUP_DIR does not exist"
    exit 1
fi
backup_name="$(date -u "+%Y-%m-%dT%H-%M-%S-%3NZ")-backup.sql.gz"
backup_path="$BACKUP_DIR/$backup_name"
backup_object="gs://$BUCKET/$backup_name"

cleanup_old_backups() {
    # Remove any incomplete backups.
    find $BACKUP_DIR -maxdepth 1 -name "*-backup.sql.gz.tmp" -type f -delete

    # Remove any old backups that weren't uploaded.
    find $BACKUP_DIR -maxdepth 1 -name "*-backup.sql.gz" -type f -mtime +7 -delete

    # TODO - Consider uploading backups that weren't previously uploaded.
    # As is, we'll accumulate backups for N days if we're unable to upload to cloud storage
    # which might fill up our disk.
}

create_backup() {
    echo "$(date -u "+%Y-%m-%dT%H:%M:%S.%3NZ"): Exporting backup to $backup_path..."
    PGPASSWORD=$PSQL_PASSWORD pg_dump \
        --host="$PSQL_HOST" \
        --port="$PSQL_PORT" \
        --db="$PSQL_DB" \
        --username="$PSQL_USER" \
        | gzip > $backup_path.tmp
    mv $backup_path.tmp $backup_path
    echo "$(date -u "+%Y-%m-%dT%H:%M:%S.%3NZ"): Finished export of size $(ls -lh $backup_path | awk -F " " {'print $5'})."
}

upload_backup() {
    echo "$(date -u "+%Y-%m-%dT%H:%M:%S.%3NZ"): Uploading backup to $backup_object..."
    gcloud storage cp $backup_path $backup_object
    echo "$(date -u "+%Y-%m-%dT%H:%M:%S.%3NZ"): Finished upload."
}

cleanup_old_backups
create_backup
upload_backup
rm $backup_path
