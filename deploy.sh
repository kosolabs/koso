#!/usr/bin/env zsh

set -e

function _on_fail {
    telegram "Failed to deploy $(git rev-parse --short HEAD) \\- $(git log --format=%s%b -n 1 HEAD | telegram_escape)" "❌"
}
trap _on_fail ZERR

source /root/.telegram.zsh

pushd frontend
npm run build
popd

pushd backend
cargo build --release
DATABASE_URL=postgresql://koso:koso@localhost/koso sqlx migrate run
popd

systemctl daemon-reload
systemctl restart koso

telegram "Deployed $(git rev-parse --short HEAD) \\- $(git log --format=%s%b -n 1 HEAD | telegram_escape)" "✅"
