#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import datetime as dt
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

DEFAULT_RUNTIME_SERVICE_ACCOUNT_NAME = "bominal-runtime"
DEFAULT_DEPLOY_SUBSCRIPTION = "bominal-deploy-requests-vm"


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
    runtime_service_account: str,
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
        f"serviceAccount:{runtime_service_account}",
        "--role",
        "roles/secretmanager.secretAccessor",
    ]
    if dry_run:
        log("INFO", f"Dry-run: would run {' '.join(cmd)}")
        return
    run_cmd(cmd)


def add_deploy_subscription_binding(
    project_id: str,
    subscription: str,
    runtime_service_account: str,
    *,
    dry_run: bool,
) -> None:
    cmd = [
        "gcloud",
        "pubsub",
        "subscriptions",
        "add-iam-policy-binding",
        subscription,
        "--project",
        project_id,
        "--member",
        f"serviceAccount:{runtime_service_account}",
        "--role",
        "roles/pubsub.subscriber",
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
        "--runtime-service-account-email",
        default="",
        help=(
            "Explicit runtime service-account email for IAM bindings "
            "(default: <runtime-service-account-name>@<project-id>.iam.gserviceaccount.com)"
        ),
    )
    parser.add_argument(
        "--runtime-service-account-name",
        default=DEFAULT_RUNTIME_SERVICE_ACCOUNT_NAME,
        help=(
            "Runtime service-account name used when --runtime-service-account-email is omitted "
            f"(default: {DEFAULT_RUNTIME_SERVICE_ACCOUNT_NAME})"
        ),
    )
    parser.add_argument(
        "--deploy-subscription",
        default=DEFAULT_DEPLOY_SUBSCRIPTION,
        help=f"Deploy Pub/Sub subscription name for subscriber IAM binding (default: {DEFAULT_DEPLOY_SUBSCRIPTION})",
    )
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
    # Backward-compatible alias for older invocations.
    parser.add_argument(
        "--vm-service-account",
        default="",
        help=argparse.SUPPRESS,
    )
    parser.add_argument("--skip-enable-api", action="store_true", help="Skip gcloud services enable")
    parser.add_argument(
        "--skip-secret-iam-binding",
        "--skip-iam-binding",
        dest="skip_secret_iam_binding",
        action="store_true",
        help="Skip Secret Manager secretAccessor IAM binding",
    )
    parser.add_argument(
        "--skip-pubsub-binding",
        action="store_true",
        help="Skip Pub/Sub subscriber IAM binding for deploy subscription",
    )
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

        runtime_sa = (args.runtime_service_account_email or args.vm_service_account or "").strip()
        if not runtime_sa:
            runtime_sa_name = (args.runtime_service_account_name or "").strip()
            if not runtime_sa_name:
                raise ScriptError(
                    "Runtime service account name is empty. Set --runtime-service-account-name "
                    "or --runtime-service-account-email."
                )
            runtime_sa = f"{runtime_sa_name}@{project_id}.iam.gserviceaccount.com"
        if "@" not in runtime_sa:
            raise ScriptError(f"Runtime service-account email is invalid: {runtime_sa}")

        if not args.skip_secret_iam_binding:
            add_secret_accessor_binding(
                project_id,
                secret_id,
                runtime_sa,
                dry_run=args.dry_run,
            )
        if not args.skip_pubsub_binding:
            add_deploy_subscription_binding(
                project_id,
                args.deploy_subscription,
                runtime_sa,
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
        log("INFO", f"Runtime service account: {runtime_sa}")
        if not args.skip_secret_iam_binding:
            log("INFO", f"secretAccessor IAM member: serviceAccount:{runtime_sa}")
        if not args.skip_pubsub_binding:
            log(
                "INFO",
                f"pubsub subscriber IAM member on {args.deploy_subscription}: serviceAccount:{runtime_sa}",
            )
        log("INFO", "Next steps:")
        log("INFO", "  1) bash infra/scripts/predeploy-check.sh --skip-smoke-tests")
        log("INFO", "  2) sudo -u bominal /opt/bominal/repo/infra/scripts/deploy.sh")
        return 0
    except ScriptError as exc:
        log("ERROR", str(exc))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
