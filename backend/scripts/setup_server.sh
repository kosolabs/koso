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
# MANUAL - add developer public keys /root/.ssh/authorized_keys
# MANUAL - add public key corresponding to environment secret KOSO_KEY to /root/.ssh/authorized_keys

# Setup github auth via deploy keys
# https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys#set-up-deploy-keys
ssh-keygen -t ed25519 -C "koso-github-read-key" -f /root/.ssh/koso_github_read_id_ed25519 -N ''
eval "$(ssh-agent -s)"
cat >>/root/.ssh/config <<EOL
Host github.com
  AddKeysToAgent yes
  IdentityFile  ~/.ssh/koso_github_read_id_ed25519
EOL
# MANUAL - add a new deploy key with the public key (e.g. ssh-ed25519 KEY) to https://github.com/kosolabs/koso/settings/keys/new
cat /root/.ssh/koso_github_read_id_ed25519.pub
ssh -T git@github.com && echo "Github auth works"

# Setup dotfiles
git clone git@github.com:shadanan/dotfiles.git .dotfiles
.dotfiles/install

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
# MANUAL - Login via a personal access token granted "read:packages". Unfortunately the only option.
# https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-to-the-container-registry
DOCKER_USER=TODO DOCKER_PULL_TOKEN=TODO echo $DOCKER_PULL_TOKEN | docker login ghcr.io -u $DOCKER_USER --password-stdin
docker pull ghcr.io/kosolabs/koso@:main && echo "Docker auth works"

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