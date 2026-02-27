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

if ! command -v uv >/dev/null 2>&1; then
  echo "==> Installing uv (Astral) to /usr/local/bin"
  curl -LsSf https://astral.sh/uv/install.sh | env UV_INSTALL_DIR=/usr/local/bin sh
fi

systemctl enable --now docker

if ! id "$APP_USER" >/dev/null 2>&1; then
  useradd --system --create-home --home "$APP_HOME" --shell /bin/bash "$APP_USER"
fi

mkdir -p "$APP_HOME"
chown -R "$APP_USER:$APP_USER" "$APP_HOME"
usermod -aG docker "$APP_USER"

# Add swap for e2-micro (1GB RAM) to prevent OOM during builds
SWAPFILE="/swapfile"
if [[ ! -f "$SWAPFILE" ]]; then
  echo "==> Creating 1GB swap file"
  fallocate -l 1G "$SWAPFILE"
  chmod 600 "$SWAPFILE"
  mkswap "$SWAPFILE"
  swapon "$SWAPFILE"
  echo "$SWAPFILE none swap sw 0 0" >> /etc/fstab
  echo "Swap enabled: $(swapon --show)"
else
  echo "==> Swap file already exists"
fi

echo "Bootstrap complete."
echo "Next:"
echo "  1) Copy repo to $APP_HOME/repo"
echo "  2) Create infra/env/prod/*.env files"
echo "  3) Run: sudo -u $APP_USER $APP_HOME/repo/infra/scripts/deploy.sh"
