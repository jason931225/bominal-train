#!/usr/bin/env bash
set -euo pipefail

SWAP_FILE="${BOMINAL_SWAP_FILE:-/swapfile}"
SWAP_SIZE="${BOMINAL_SWAP_SIZE:-2G}"
SWAPPINESS="${BOMINAL_SWAPPINESS:-10}"
VFS_CACHE_PRESSURE="${BOMINAL_VFS_CACHE_PRESSURE:-50}"
SYSCTL_FILE="${BOMINAL_SYSCTL_FILE:-/etc/sysctl.d/99-bominal-swap.conf}"

run_root() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  else
    sudo "$@"
  fi
}

ensure_swap() {
  if ! run_root /sbin/swapon --show=NAME --noheadings | grep -qx "${SWAP_FILE}"; then
    if [ ! -f "${SWAP_FILE}" ]; then
      run_root fallocate -l "${SWAP_SIZE}" "${SWAP_FILE}" || run_root dd if=/dev/zero of="${SWAP_FILE}" bs=1M count=2048 status=none
    fi
    run_root chmod 600 "${SWAP_FILE}"
    run_root mkswap "${SWAP_FILE}" >/dev/null
    run_root /sbin/swapon "${SWAP_FILE}"
  fi

  if ! run_root grep -qE "^${SWAP_FILE}[[:space:]]" /etc/fstab; then
    run_root sh -c "echo '${SWAP_FILE} none swap sw 0 0' >> /etc/fstab"
  fi
}

ensure_sysctl() {
  run_root sh -c "printf 'vm.swappiness=%s\nvm.vfs_cache_pressure=%s\n' '${SWAPPINESS}' '${VFS_CACHE_PRESSURE}' > '${SYSCTL_FILE}'"
  run_root /sbin/sysctl -w "vm.swappiness=${SWAPPINESS}" >/dev/null
  run_root /sbin/sysctl -w "vm.vfs_cache_pressure=${VFS_CACHE_PRESSURE}" >/dev/null
}

ensure_swap
ensure_sysctl
