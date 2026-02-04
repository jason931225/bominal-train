#!/usr/bin/env bash
set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Run as root: sudo ./infra/scripts/vm-docker-bootstrap.sh"
  exit 1
fi

APP_USER="${APP_USER:-bominal}"
APP_HOME="${APP_HOME:-/opt/bominal}"

echo "==> Installing base packages"
apt-get update
apt-get install -y ca-certificates curl git

if ! command -v docker >/dev/null 2>&1; then
  echo "==> Installing Docker Engine + Compose plugin"
  curl -fsSL https://get.docker.com | sh
fi

systemctl enable --now docker

if ! id "$APP_USER" >/dev/null 2>&1; then
  useradd --system --create-home --home "$APP_HOME" --shell /bin/bash "$APP_USER"
fi

mkdir -p "$APP_HOME"
chown -R "$APP_USER:$APP_USER" "$APP_HOME"
usermod -aG docker "$APP_USER"

echo "Bootstrap complete."
echo "Next:"
echo "  1) Copy repo to $APP_HOME/repo"
echo "  2) Create infra/env/prod/*.env files"
echo "  3) Run: sudo -u $APP_USER $APP_HOME/repo/infra/scripts/vm-docker-deploy.sh latest"
