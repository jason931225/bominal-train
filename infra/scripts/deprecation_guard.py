#!/usr/bin/env python3
"""
Registry-driven deprecation policy checks for local, CI, and deploy gates.
"""

from __future__ import annotations

import argparse
import datetime as dt
import fnmatch
import json
import sys
from pathlib import Path
from typing import Dict, Iterable, List, Sequence, Tuple


VALID_SURFACES = {"runtime", "ci", "docs", "api", "env", "script", "mixed"}
VALID_SCOPES = {"production", "github", "local", "mixed"}
VALID_STATUSES = {"deprecated", "removal_scheduled", "removed", "blocked"}
WINDOW_POLICY = "prod30_github14_local2"
GLOB_CHARS = set("*?[]")


def parse_args() -> argparse.Namespace:
  parser = argparse.ArgumentParser(description="Deprecation workflow guard checks")
  subparsers = parser.add_subparsers(dest="command", required=True)

  for cmd in ("validate", "scan-active-references", "enforce-deploy"):
    sub = subparsers.add_parser(cmd)
    sub.add_argument("--root", default=".", help="Repository root (default: cwd)")
    sub.add_argument(
      "--registry",
      help="Registry path (default: <root>/docs/deprecations/registry.json)",
    )
    sub.add_argument(
      "--today",
      help="Override current date (YYYY-MM-DD), useful for deterministic tests",
    )

  return parser.parse_args()


def fail(message: str) -> None:
  print(f"ERROR: {message}", file=sys.stderr)
  raise SystemExit(1)


def parse_date(raw: str, field_name: str, entry_id: str) -> dt.date:
  try:
    return dt.date.fromisoformat(raw)
  except ValueError:
    fail(f"{entry_id}: invalid {field_name} date (expected YYYY-MM-DD): {raw}")


def normalize_rel(rel_path: Path) -> str:
  return rel_path.as_posix()


def has_glob(pattern: str) -> bool:
  return any(ch in pattern for ch in GLOB_CHARS)


def matches_allow_pattern(rel_path: str, pattern: str) -> bool:
  if has_glob(pattern):
    return fnmatch.fnmatch(rel_path, pattern)
  normalized = pattern.rstrip("/")
  return rel_path == normalized or rel_path.startswith(f"{normalized}/")


def resolve_pattern_targets(root: Path, pattern: str) -> List[Path]:
  if pattern.startswith("/"):
    fail(f"Invalid callers_scan_paths entry (absolute path disallowed): {pattern}")
  if ".." in Path(pattern).parts:
    fail(f"Invalid callers_scan_paths entry (parent traversal disallowed): {pattern}")

  if has_glob(pattern):
    return sorted(root.glob(pattern))
  return [root / pattern]


def iter_scan_files(target: Path) -> Iterable[Path]:
  if target.is_file():
    yield target
    return
  if target.is_dir():
    for path in sorted(target.rglob("*")):
      if path.is_file():
        if ".git" in path.parts:
          continue
        yield path


def load_registry(root: Path, registry_arg: str | None) -> Tuple[Path, Dict]:
  registry_path = Path(registry_arg) if registry_arg else root / "docs/deprecations/registry.json"
  if not registry_path.is_absolute():
    registry_path = root / registry_path

  if not registry_path.is_file():
    fail(f"Registry file not found: {registry_path}")

  try:
    payload = json.loads(registry_path.read_text(encoding="utf-8"))
  except json.JSONDecodeError as exc:
    fail(f"Registry is not valid JSON: {registry_path}: {exc}")

  if not isinstance(payload, dict):
    fail("Registry payload must be a JSON object.")
  return registry_path, payload


def validate_entry_types(entry: Dict, entry_id: str) -> None:
  required_str_fields = [
    "id",
    "surface",
    "scope",
    "artifact",
    "replacement",
    "owner",
    "status",
    "deprecated_on",
    "remove_after",
    "window_policy",
    "notes",
  ]
  for field in required_str_fields:
    value = entry.get(field)
    if not isinstance(value, str) or not value.strip():
      fail(f"{entry_id}: missing or empty required field: {field}")

  scan_paths = entry.get("callers_scan_paths")
  if not isinstance(scan_paths, list) or not scan_paths:
    fail(f"{entry_id}: callers_scan_paths must be a non-empty list")
  for value in scan_paths:
    if not isinstance(value, str) or not value.strip():
      fail(f"{entry_id}: callers_scan_paths entries must be non-empty strings")

  allow_paths = entry.get("allow_reference_paths")
  if allow_paths is not None:
    if not isinstance(allow_paths, list):
      fail(f"{entry_id}: allow_reference_paths must be a list when present")
    for value in allow_paths:
      if not isinstance(value, str) or not value.strip():
        fail(f"{entry_id}: allow_reference_paths entries must be non-empty strings")


def validate_window_policy(entry: Dict, entry_id: str, deprecated_on: dt.date, remove_after: dt.date) -> None:
  if entry["window_policy"] != WINDOW_POLICY:
    fail(
      f"{entry_id}: unsupported window_policy {entry['window_policy']!r} "
      f"(expected {WINDOW_POLICY!r})"
    )

  has_exception = entry.get("window_exception_approved", False) is True
  if has_exception:
    reason = entry.get("window_exception_reason")
    if not isinstance(reason, str) or not reason.strip():
      fail(f"{entry_id}: window_exception_reason is required when exception is approved")

  minimum_days = 0
  scope = entry["scope"]
  if scope == "production":
    minimum_days = 30
  elif scope == "github":
    minimum_days = 14
  elif scope == "mixed":
    minimum_days = 30

  if minimum_days > 0:
    delta = (remove_after - deprecated_on).days
    if delta < minimum_days and not has_exception:
      fail(
        f"{entry_id}: remove_after must be at least {minimum_days} days after deprecated_on "
        f"for scope={scope} (or provide approved window exception)"
      )

  if scope == "local":
    required = entry.get("local_release_cycles_required")
    completed = entry.get("local_release_cycles_completed")
    if not isinstance(required, int) or required < 2:
      fail(f"{entry_id}: local_release_cycles_required must be an integer >= 2")
    if not isinstance(completed, int) or completed < 0:
      fail(f"{entry_id}: local_release_cycles_completed must be an integer >= 0")


def validate_registry(root: Path, payload: Dict) -> List[Dict]:
  schema_version = payload.get("schema_version")
  if schema_version != 1:
    fail(f"Unsupported schema_version: {schema_version!r} (expected 1)")

  generated_at = payload.get("generated_at")
  if not isinstance(generated_at, str):
    fail("generated_at must be a YYYY-MM-DD string")
  parse_date(generated_at, "generated_at", "registry")

  entries = payload.get("deprecations")
  if not isinstance(entries, list) or not entries:
    fail("deprecations must be a non-empty list")

  seen_ids = set()
  for index, raw_entry in enumerate(entries):
    if not isinstance(raw_entry, dict):
      fail(f"deprecations[{index}] must be an object")
    entry = raw_entry
    entry_id = entry.get("id", f"deprecations[{index}]")
    validate_entry_types(entry, str(entry_id))

    if entry["id"] in seen_ids:
      fail(f"Duplicate deprecation id: {entry['id']}")
    seen_ids.add(entry["id"])

    if entry["surface"] not in VALID_SURFACES:
      fail(f"{entry_id}: invalid surface {entry['surface']!r}")
    if entry["scope"] not in VALID_SCOPES:
      fail(f"{entry_id}: invalid scope {entry['scope']!r}")
    if entry["status"] not in VALID_STATUSES:
      fail(f"{entry_id}: invalid status {entry['status']!r}")

    deprecated_on = parse_date(entry["deprecated_on"], "deprecated_on", entry_id)
    remove_after = parse_date(entry["remove_after"], "remove_after", entry_id)
    if remove_after < deprecated_on:
      fail(f"{entry_id}: remove_after cannot be earlier than deprecated_on")

    validate_window_policy(entry, entry_id, deprecated_on, remove_after)

    if entry["status"] == "removed":
      removed_on_raw = entry.get("removed_on")
      if not isinstance(removed_on_raw, str):
        fail(f"{entry_id}: removed_on is required when status=removed")
      removed_on = parse_date(removed_on_raw, "removed_on", entry_id)
      if removed_on < deprecated_on:
        fail(f"{entry_id}: removed_on cannot be earlier than deprecated_on")
      if entry["scope"] == "local":
        required = int(entry["local_release_cycles_required"])
        completed = int(entry["local_release_cycles_completed"])
        if completed < required:
          fail(
            f"{entry_id}: local removal requires local_release_cycles_completed >= "
            "local_release_cycles_required"
          )
    else:
      if "removed_on" in entry:
        fail(f"{entry_id}: removed_on is only allowed when status=removed")

    for pattern in entry["callers_scan_paths"]:
      targets = resolve_pattern_targets(root, pattern)
      if not any(target.exists() for target in targets):
        fail(f"{entry_id}: callers_scan_paths target does not exist: {pattern}")

  return entries


def find_artifact_references(root: Path, entry: Dict) -> List[str]:
  artifact = entry["artifact"]
  allow_patterns = entry.get("allow_reference_paths", [])
  refs: List[str] = []
  seen_files = set()

  for pattern in entry["callers_scan_paths"]:
    for target in resolve_pattern_targets(root, pattern):
      for file_path in iter_scan_files(target):
        try:
          rel_path = file_path.relative_to(root)
        except ValueError:
          continue
        rel = normalize_rel(rel_path)
        if rel in seen_files:
          continue
        seen_files.add(rel)

        if allow_patterns and any(matches_allow_pattern(rel, p) for p in allow_patterns):
          continue

        try:
          content = file_path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
          continue

        pos = content.find(artifact)
        if pos == -1:
          continue

        line = content.count("\n", 0, pos) + 1
        refs.append(f"{rel}:{line}")

  return refs


def collect_violations(entries: Sequence[Dict], root: Path, today: dt.date, deploy_only: bool) -> List[str]:
  violations: List[str] = []

  for entry in entries:
    entry_id = entry["id"]
    scope = entry["scope"]
    status = entry["status"]

    if deploy_only and scope not in {"production", "mixed"}:
      continue

    remove_after = parse_date(entry["remove_after"], "remove_after", entry_id)
    refs = find_artifact_references(root, entry)

    if status == "removed":
      if refs:
        violations.append(
          f"{entry_id}: removed artifact still referenced in active paths: "
          f"{', '.join(refs)}"
        )
      continue

    if today > remove_after:
      if refs:
        violations.append(
          f"{entry_id}: deprecation deadline passed ({remove_after}) and active references remain: "
          f"{', '.join(refs)}"
        )
      else:
        violations.append(
          f"{entry_id}: deprecation deadline passed ({remove_after}) but status is still "
          f"{status!r}; finalize removal or update registry with approved exception"
        )

  return violations


def resolve_today(raw_today: str | None) -> dt.date:
  if raw_today:
    return dt.date.fromisoformat(raw_today)
  return dt.date.today()


def main() -> int:
  args = parse_args()
  root = Path(args.root).resolve()
  if not root.exists():
    fail(f"Repository root does not exist: {root}")

  registry_path, payload = load_registry(root, args.registry)
  entries = validate_registry(root, payload)
  today = resolve_today(args.today)

  if args.command == "validate":
    print(f"OK: deprecation registry is valid ({registry_path})")
    return 0

  deploy_only = args.command == "enforce-deploy"
  violations = collect_violations(entries, root, today, deploy_only=deploy_only)
  if violations:
    for violation in violations:
      print(f"ERROR: {violation}", file=sys.stderr)
    return 1

  if args.command == "scan-active-references":
    print("OK: deprecation reference policy checks passed.")
    return 0

  if args.command == "enforce-deploy":
    print("OK: deprecation deploy gate checks passed.")
    return 0

  fail(f"Unsupported command: {args.command}")
  return 1


if __name__ == "__main__":
  raise SystemExit(main())
