#!/bin/bash
set -x

git fetch
if git merge-base --is-ancestor origin/main main; then
  exit 0
fi
git pull

pushd frontend
npm run build
popd

pushd backend
cargo build --release
DATABASE_URL=postgresql://koso:koso@localhost/koso sqlx migrate run
popd

systemctl daemon-reload
systemctl restart koso
