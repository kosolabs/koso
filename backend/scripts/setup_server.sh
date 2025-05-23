#!/usr/bin/env zsh

set -eo pipefail

# NOTE - this script was pieced together from history and is currently untested.
# Be long the look out for MANUAL steps. Good luck

cd /root

# Install tools
sudo apt update
sudo apt upgrade
apt install git tmux zsh vim unattended-upgrades

# Setup SSH auth
cat >> /root/.ssh/authorized_keys <<EOL
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFES++sWzKktYuyLQIwhnIfJX75OheYvTbp16G6rF97u shad@storm
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIM1Vu2l/fbZgXt94US7PWgxTagVymWmaeB4zLM8cQuyl kyle@mac
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINiVwOyxjtYb3dNx40mEZ8se6JRPSF6OE2VFVmryEJRf Koso-Deploy-Key-KOSO_KEY
EOL

# Setup dotfiles
git clone https://github.com/shadanan/dotfiles.git .dotfiles
.dotfiles/install

# Enable unattended upgrades
sudo apt update
sudo apt install unattended-upgrades

cat >> /etc/apt/apt.conf.d/20auto-upgrades << EOL
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
APT::Periodic::AutocleanInterval "7";
EOL
sudo unattended-upgrades --dry-run --debug

# Clone the Koso repo
git clone https://github.com/kosolabs/koso.git

# Install Docker
sudo su &&\
apt update &&\
apt install ca-certificates curl gnupg apt-transport-https gpg
curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /usr/share/keyrings/docker.gpg
apt update
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker.gpg] https://download.docker.com/linux/debian bookworm stable" |tee /etc/apt/sources.list.d/docker.list > /dev/null
apt install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin docker-compose
systemctl status docker
systemctl is-active docker && echo "Docker running"

# Install caddy
sudo apt update
apt install caddy
cat >>/etc/caddy/Caddyfile <<EOL

# Koso server config
koso.app {
  reverse_proxy 127.0.0.1:3000
}
EOL
systemctl restart caddy.service

# Install Postgresql
sudo apt update
sudo apt install -y postgresql-common
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh
apt install postgresql-17
cat >>/etc/postgresql/16/main/postgresql.conf <<EOL

# Commit asynchronously for better performance
synchronous_commit = off
EOL

systemctl start postgresql.service
systemctl enable postgresql.service
systemctl status postgresql.service
systemctl is-active postgresql.service && echo "Postgresql running"
docker pull ghcr.io/kosolabs/koso@:main
docker run \
    --env DATABASE_URL=postgresql://koso:koso@localhost/koso \
    --network=host \
    --rm \
    ghcr.io/kosolabs/koso:main \
    "./sqlx" database create

# Install gcloud
# TODO: Consider https://cloud.google.com/sdk/docs/downloads-versioned-archives
sudo apt update
sudo apt-get install apt-transport-https ca-certificates gnupg curl
curl https://packages.cloud.google.com/apt/doc/apt-key.gpg | sudo gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg
echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" | sudo tee -a /etc/apt/sources.list.d/google-cloud-sdk.list
sudo apt-get update && sudo apt-get install google-cloud-cli
# Manual - auth with a SA
gcloud auth activate-service-account --project=$PROJECT --key-file=sa.json

# Create secrets
mkdir -p /root/.secrets/github
# The Github app's key
touch /root/.secrets/github/key.pem
# The Github app's webhook secret
touch /root/.secrets/github/webhook_secret
# The Github app's client secret
touch /root/.secrets/github/client_secret
# The telegram bot token
touch /root/.secrets/telegram/token
# The pem encoded RSA key used to sign telegram auth tokens
touch /root/.secrets/koso/hmac
# MANUAL - place secrets in those files

# Finally, run the Deploy action to start the backend.
# https://github.com/kosolabs/koso/actions/workflows/deploy.yml