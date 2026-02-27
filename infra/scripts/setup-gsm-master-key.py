#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import datetime as dt
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib import error, request

METADATA_VM_SA_URL = (
    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/email"
)


class ScriptError(RuntimeError):
    """Raised when setup-gsm-master-key cannot complete safely."""


def log(level: str, message: str) -> None:
    print(f"[{level}] {message}")


def run_cmd(
    cmd: list[str],
    *,
    check: bool = True,
    input_text: str | None = None,
) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        cmd,
        input=input_text,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if check and completed.returncode != 0:
        stderr = (completed.stderr or "").strip()
        stdout = (completed.stdout or "").strip()
        detail = stderr or stdout or "unknown error"
        raise ScriptError(f"Command failed ({' '.join(cmd)}): {detail}")
    return completed


def require_tool(name: str) -> None:
    if shutil.which(name):
        return
    raise ScriptError(f"Required command is not available: {name}")


def read_env_key(env_path: Path, key: str) -> str:
    if not env_path.exists():
        return ""
    for raw_line in env_path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in raw_line:
            continue
        k, v = raw_line.split("=", 1)
        if k.strip() != key:
            continue
        value = v.strip()
        if (value.startswith('"') and value.endswith('"')) or (
            value.startswith("'") and value.endswith("'")
        ):
            value = value[1:-1]
        return value
    return ""


def validate_master_key_b64(master_key: str) -> str:
    normalized = (master_key or "").strip()
    if not normalized:
        raise ScriptError("MASTER_KEY is empty. Set MASTER_KEY in infra/env/prod/api.env or pass --master-key.")
    try:
        decoded = base64.b64decode(normalized.encode("utf-8"), validate=True)
    except Exception as exc:  # pragma: no cover - exact exception type not important
        raise ScriptError("MASTER_KEY is not valid base64.") from exc
    if len(decoded) != 32:
        raise ScriptError(f"MASTER_KEY must decode to 32 bytes (got {len(decoded)} bytes).")
    return normalized


def fetch_vm_service_account_email() -> str:
    req = request.Request(
        METADATA_VM_SA_URL,
        headers={"Metadata-Flavor": "Google"},
        method="GET",
    )
    try:
        with request.urlopen(req, timeout=5.0) as response:
            payload = response.read().decode("utf-8").strip()
    except error.URLError as exc:
        raise ScriptError(
            "Unable to resolve VM service account from metadata server. "
            "Pass --vm-service-account explicitly."
        ) from exc
    if not payload:
        raise ScriptError(
            "Metadata server returned an empty VM service account. "
            "Pass --vm-service-account explicitly."
        )
    return payload


def ensure_secret_api_enabled(project_id: str, *, dry_run: bool) -> None:
    cmd = ["gcloud", "services", "enable", "secretmanager.googleapis.com", "--project", project_id]
    if dry_run:
        log("INFO", f"Dry-run: would run {' '.join(cmd)}")
        return
    run_cmd(cmd)


def ensure_secret_exists(project_id: str, secret_id: str, *, dry_run: bool) -> None:
    describe_cmd = ["gcloud", "secrets", "describe", secret_id, "--project", project_id]
    describe_result = run_cmd(describe_cmd, check=False)
    if describe_result.returncode == 0:
        return

    create_cmd = [
        "gcloud",
        "secrets",
        "create",
        secret_id,
        "--replication-policy=automatic",
        "--project",
        project_id,
    ]
    if dry_run:
        log("INFO", f"Dry-run: would run {' '.join(create_cmd)}")
        return

    run_cmd(create_cmd)


def latest_secret_version(project_id: str, secret_id: str) -> str:
    list_cmd = [
        "gcloud",
        "secrets",
        "versions",
        "list",
        secret_id,
        "--project",
        project_id,
        "--sort-by=~createTime",
        "--limit=1",
        "--format=value(name)",
    ]
    result = run_cmd(list_cmd)
    value = (result.stdout or "").strip()
    if not value:
        raise ScriptError(
            "Could not determine latest secret version from Secret Manager after write/list operation."
        )
    version = value.rsplit("/", 1)[-1].strip()
    if not version or version.lower() == "latest":
        raise ScriptError("Resolved secret version is invalid; expected a pinned numeric version.")
    return version


def add_secret_version(project_id: str, secret_id: str, master_key_b64: str, *, dry_run: bool) -> str:
    add_cmd = [
        "gcloud",
        "secrets",
        "versions",
        "add",
        secret_id,
        "--data-file=-",
        "--project",
        project_id,
    ]
    if dry_run:
        log("INFO", f"Dry-run: would run {' '.join(add_cmd)}")
        log("INFO", "Dry-run: would list latest secret version to pin GSM_MASTER_KEY_VERSION")
        return "CHANGE_ME_PINNED_VERSION"

    run_cmd(add_cmd, input_text=master_key_b64)
    return latest_secret_version(project_id, secret_id)


def verify_secret_version_exists(project_id: str, secret_id: str, version: str, *, dry_run: bool) -> None:
    cmd = [
        "gcloud",
        "secrets",
        "versions",
        "describe",
        version,
        "--secret",
        secret_id,
        "--project",
        project_id,
    ]
    if dry_run:
        log("INFO", f"Dry-run: would run {' '.join(cmd)}")
        return
    run_cmd(cmd)


def add_secret_accessor_binding(
    project_id: str,
    secret_id: str,
    vm_service_account: str,
    *,
    dry_run: bool,
) -> None:
    cmd = [
        "gcloud",
        "secrets",
        "add-iam-policy-binding",
        secret_id,
        "--project",
        project_id,
        "--member",
        f"serviceAccount:{vm_service_account}",
        "--role",
        "roles/secretmanager.secretAccessor",
    ]
    if dry_run:
        log("INFO", f"Dry-run: would run {' '.join(cmd)}")
        return
    run_cmd(cmd)


def backup_file(path: Path) -> Path:
    ts = dt.datetime.utcnow().strftime("%Y%m%d%H%M%S")
    backup_path = path.with_name(f"{path.name}.bak.{ts}")
    shutil.copy2(path, backup_path)
    return backup_path


def upsert_env_keys(env_path: Path, updates: dict[str, str], *, dry_run: bool) -> None:
    original = env_path.read_text(encoding="utf-8")
    lines = original.splitlines()

    seen: set[str] = set()
    updated_lines: list[str] = []
    key_pattern = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$")

    for line in lines:
        match = key_pattern.match(line)
        if match:
            key = match.group(1)
            if key in updates and key not in seen:
                updated_lines.append(f"{key}={updates[key]}")
                seen.add(key)
                continue
        updated_lines.append(line)

    for key, value in updates.items():
        if key in seen:
            continue
        updated_lines.append(f"{key}={value}")

    rendered = "\n".join(updated_lines)
    if rendered and not rendered.endswith("\n"):
        rendered += "\n"

    if dry_run:
        log("INFO", "Dry-run: would update the following keys in api.env:")
        for key, value in updates.items():
            log("INFO", f"  {key}={value}")
        return

    with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=env_path.parent, delete=False) as tmp_file:
        tmp_file.write(rendered)
        tmp_path = Path(tmp_file.name)
    tmp_path.replace(env_path)


def parse_args() -> argparse.Namespace:
    script_dir = Path(__file__).resolve().parent
    default_root = Path(os.environ.get("BOMINAL_ROOT_DIR", str(script_dir.parent.parent)))

    parser = argparse.ArgumentParser(
        description="Configure bominal production env to use GCP Secret Manager for MASTER_KEY."
    )
    parser.add_argument("--root-dir", default=str(default_root), help="Repository root directory")
    parser.add_argument(
        "--api-env",
        default="",
        help="Path to production api.env (default: <root>/infra/env/prod/api.env)",
    )
    parser.add_argument("--project-id", default="", help="GCP project id (fallback: api.env GCP_PROJECT_ID)")
    parser.add_argument("--secret-id", default="bominal-master-key", help="Secret Manager secret id")
    parser.add_argument(
        "--master-key",
        default="",
        help="Base64 32-byte master key payload (fallback: api.env MASTER_KEY)",
    )
    parser.add_argument(
        "--pin-version",
        default="",
        help="Existing secret version to pin. If omitted, script adds a new secret version from MASTER_KEY.",
    )
    parser.add_argument(
        "--vm-service-account",
        default="",
        help="VM service-account email for secretAccessor binding (fallback: metadata server)",
    )
    parser.add_argument("--skip-enable-api", action="store_true", help="Skip gcloud services enable")
    parser.add_argument("--skip-iam-binding", action="store_true", help="Skip IAM binding step")
    parser.add_argument("--dry-run", action="store_true", help="Preview actions without mutating state")
    parser.add_argument("--no-backup", action="store_true", help="Skip api.env backup before writing")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root_dir = Path(args.root_dir).resolve()
    api_env = Path(args.api_env).resolve() if args.api_env else root_dir / "infra" / "env" / "prod" / "api.env"

    try:
        require_tool("gcloud")

        if not api_env.exists():
            raise ScriptError(f"Production api.env not found: {api_env}")

        project_id = (args.project_id or read_env_key(api_env, "GCP_PROJECT_ID") or "").strip()
        if not project_id:
            raise ScriptError("Missing project id. Pass --project-id or set GCP_PROJECT_ID in api.env.")

        secret_id = (args.secret_id or "").strip()
        if not secret_id:
            raise ScriptError("Secret id must be non-empty.")

        pin_version = (args.pin_version or "").strip()
        if pin_version and pin_version.lower() == "latest":
            raise ScriptError("--pin-version cannot be 'latest'. Use a concrete pinned version.")

        master_key_b64 = ""
        if not pin_version:
            master_key_b64 = validate_master_key_b64(args.master_key or read_env_key(api_env, "MASTER_KEY"))

        if not args.skip_enable_api:
            ensure_secret_api_enabled(project_id, dry_run=args.dry_run)

        ensure_secret_exists(project_id, secret_id, dry_run=args.dry_run)

        if pin_version:
            verify_secret_version_exists(project_id, secret_id, pin_version, dry_run=args.dry_run)
            pinned_version = pin_version
        else:
            pinned_version = add_secret_version(project_id, secret_id, master_key_b64, dry_run=args.dry_run)

        vm_sa = ""
        if not args.skip_iam_binding:
            vm_sa = (args.vm_service_account or "").strip()
            if not vm_sa:
                if args.dry_run:
                    vm_sa = "CHANGE_ME_VM_SERVICE_ACCOUNT"
                    log(
                        "WARN",
                        "Dry-run: metadata lookup skipped for VM service account. "
                        "Pass --vm-service-account for exact IAM preview.",
                    )
                else:
                    vm_sa = fetch_vm_service_account_email()
            add_secret_accessor_binding(
                project_id,
                secret_id,
                vm_sa,
                dry_run=args.dry_run,
            )

        updates = {
            "GCP_PROJECT_ID": project_id,
            "GSM_MASTER_KEY_ENABLED": "true",
            "GSM_MASTER_KEY_PROJECT_ID": project_id,
            "GSM_MASTER_KEY_SECRET_ID": secret_id,
            "GSM_MASTER_KEY_VERSION": pinned_version,
            "GSM_MASTER_KEY_ALLOW_ENV_FALLBACK": "false",
        }

        if not args.dry_run and not args.no_backup:
            backup_path = backup_file(api_env)
            log("INFO", f"Backed up api.env to {backup_path}")

        upsert_env_keys(api_env, updates, dry_run=args.dry_run)

        log("OK", "GSM master-key setup complete.")
        log("INFO", f"api.env: {api_env}")
        log("INFO", f"Pinned version: {pinned_version}")
        if vm_sa:
            log("INFO", f"secretAccessor IAM member: serviceAccount:{vm_sa}")
        log("INFO", "Next steps:")
        log("INFO", "  1) bash infra/scripts/predeploy-check.sh --skip-smoke-tests")
        log("INFO", "  2) sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh")
        return 0
    except ScriptError as exc:
        log("ERROR", str(exc))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
