#!/usr/bin/env zsh

set -e

# NOTE - this script was pieced together from history and is currently untested.
# Be long the look out for MANUAL steps. Good luck

cd /root

# Install tools
sudo apt update
sudo apt upgrade
apt install git tmux zsh vim

# Setup SSH auth
cat >> /root/.ssh/authorized_keys <<EOL
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFES++sWzKktYuyLQIwhnIfJX75OheYvTbp16G6rF97u shad@storm
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIM1Vu2l/fbZgXt94US7PWgxTagVymWmaeB4zLM8cQuyl kyle@mac
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINiVwOyxjtYb3dNx40mEZ8se6JRPSF6OE2VFVmryEJRf Koso-Deploy-Key-KOSO_KEY
EOL

# Setup dotfiles
git clone https://github.com/shadanan/dotfiles.git .dotfiles
.dotfiles/install

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
apt install postgresql-16
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

# Finally, run the Deploy action to start the backend.
# https://github.com/kosolabs/koso/actions/workflows/deploy.yml