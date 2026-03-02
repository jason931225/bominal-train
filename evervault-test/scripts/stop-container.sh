#!/usr/bin/env bash
set -euo pipefail

CONTAINER_NAME="${EVERVAULT_TEST_CONTAINER_NAME:-evervault-test}"

if docker ps -a --format '{{.Names}}' | grep -qx "$CONTAINER_NAME"; then
  docker rm -f "$CONTAINER_NAME"
  echo "Stopped and removed $CONTAINER_NAME"
else
  echo "Container $CONTAINER_NAME not found"
fi
