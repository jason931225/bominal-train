#!/usr/bin/env bash
set -euo pipefail

echo "=============================================="
echo "🚀 Starting Deployment"
echo "=============================================="
echo "Commit: ${DEPLOY_COMMIT:-unknown} (${DEPLOY_SHORT:-unknown})"
echo "Actor:  ${GH_ACTOR:-unknown}"
echo "Time:   $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
echo ""

cd /opt/bominal/repo
git fetch origin

# ensure script is exactly what's on main
git checkout origin/main -- infra/scripts/deploy-zero-downtime.sh
chmod +x /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh

bash /opt/bominal/repo/infra/scripts/deploy-zero-downtime.sh "${DEPLOY_SHORT}"
