#!/usr/bin/env zsh

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

telegram "Deployed $(git rev-parse --short HEAD) \\- $(git log --format=%s%b -n 1 HEAD | telegram_escape)"
