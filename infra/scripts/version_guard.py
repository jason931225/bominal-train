#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

ROOT_DIR = Path(__file__).resolve().parents[2]
VERSION_MAP_PATH = ROOT_DIR / "docs" / "releases" / "version-map.json"
SEMVER_RE = re.compile(r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$")
VALID_BUMPS = {"major", "minor", "patch"}


class VersionGuardError(RuntimeError):
    pass


@dataclass(frozen=True)
class Release:
    version: tuple[int, int, int]
    version_text: str
    commit: str
    bump: str


def _run_git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise VersionGuardError(
            f"git {' '.join(args)} failed with code {result.returncode}: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def _is_ancestor(ancestor: str, descendant: str) -> bool:
    result = subprocess.run(
        ["git", "merge-base", "--is-ancestor", ancestor, descendant],
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    return result.returncode == 0


def _parse_semver(value: str) -> tuple[int, int, int]:
    match = SEMVER_RE.match(value)
    if not match:
        raise VersionGuardError(f"Invalid semantic version: {value!r}")
    return tuple(int(part) for part in match.groups())


def _expected_next(prev: tuple[int, int, int], bump: str) -> tuple[int, int, int]:
    major, minor, patch = prev
    if bump == "patch":
        return major, minor, patch + 1
    if bump == "minor":
        return major, minor + 1, 0
    if bump == "major":
        return major + 1, 0, 0
    raise VersionGuardError(f"Unknown bump type: {bump!r}")


def _format_version(version: tuple[int, int, int]) -> str:
    return f"{version[0]}.{version[1]}.{version[2]}"


def _to_full_commit(ref: str) -> str:
    commit = _run_git("rev-parse", "--verify", f"{ref}^{{commit}}")
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise VersionGuardError(f"Invalid commit hash resolved from {ref!r}: {commit!r}")
    return commit


def _load_version_map() -> dict[str, Any]:
    if not VERSION_MAP_PATH.exists():
        raise VersionGuardError(f"Missing version map: {VERSION_MAP_PATH}")
    with VERSION_MAP_PATH.open("r", encoding="utf-8") as handle:
        payload = json.load(handle)
    if not isinstance(payload, dict):
        raise VersionGuardError("version-map.json must be a JSON object")
    return payload


def _validated_state() -> tuple[dict[str, Any], list[Release]]:
    payload = _load_version_map()

    schema_version = payload.get("schema_version")
    if schema_version != 1:
        raise VersionGuardError(f"Unsupported schema_version: {schema_version!r} (expected 1)")

    baseline = payload.get("baseline")
    if not isinstance(baseline, dict):
        raise VersionGuardError("Missing object field: baseline")

    baseline_version_text = baseline.get("version")
    baseline_commit_ref = baseline.get("commit")
    if not isinstance(baseline_version_text, str) or not isinstance(baseline_commit_ref, str):
        raise VersionGuardError("baseline.version and baseline.commit must be strings")

    baseline_version = _parse_semver(baseline_version_text)
    baseline_commit = _to_full_commit(baseline_commit_ref)

    releases_raw = payload.get("releases")
    if not isinstance(releases_raw, list) or not releases_raw:
        raise VersionGuardError("releases must be a non-empty array")

    releases: list[Release] = []
    seen_versions: set[tuple[int, int, int]] = set()
    seen_commits: set[str] = set()

    for idx, raw_release in enumerate(releases_raw):
        if not isinstance(raw_release, dict):
            raise VersionGuardError(f"Release entry at index {idx} must be an object")

        version_text = raw_release.get("version")
        commit_ref = raw_release.get("commit")
        bump = raw_release.get("bump")
        if not isinstance(version_text, str) or not isinstance(commit_ref, str) or not isinstance(bump, str):
            raise VersionGuardError(
                f"Release entry {idx} must include string fields: version, commit, bump"
            )
        if bump not in VALID_BUMPS:
            raise VersionGuardError(
                f"Release entry {idx} has invalid bump {bump!r}; expected one of {sorted(VALID_BUMPS)}"
            )

        version_tuple = _parse_semver(version_text)
        commit = _to_full_commit(commit_ref)

        if version_tuple in seen_versions:
            raise VersionGuardError(f"Duplicate release version found: {version_text}")
        if commit in seen_commits:
            raise VersionGuardError(f"Duplicate release commit found: {commit}")

        release = Release(
            version=version_tuple,
            version_text=version_text,
            commit=commit,
            bump=bump,
        )

        if idx == 0:
            if release.version != baseline_version:
                raise VersionGuardError(
                    "First release version must match baseline.version"
                )
            if release.commit != baseline_commit:
                raise VersionGuardError(
                    "First release commit must match baseline.commit"
                )
        else:
            previous = releases[idx - 1]
            if not _is_ancestor(previous.commit, release.commit):
                raise VersionGuardError(
                    f"Release commit order invalid: {previous.commit} is not ancestor of {release.commit}"
                )
            expected_version = _expected_next(previous.version, release.bump)
            if release.version != expected_version:
                raise VersionGuardError(
                    f"Release {version_text} violates bump rule {release.bump!r}; "
                    f"expected {_format_version(expected_version)}"
                )

        seen_versions.add(version_tuple)
        seen_commits.add(commit)
        releases.append(release)

    head_commit = _to_full_commit("HEAD")
    if not _is_ancestor(baseline_commit, head_commit):
        raise VersionGuardError(
            f"Baseline commit {baseline_commit} must stay in HEAD ancestry {head_commit}"
        )

    # Normalize resolved baseline commit back into payload for downstream callers.
    payload["baseline"]["commit"] = baseline_commit
    return payload, releases


def validate() -> None:
    _, releases = _validated_state()
    latest = releases[-1]
    print(
        f"OK: version map valid ({len(releases)} release entries, latest=v{latest.version_text}@{latest.commit[:12]})"
    )


def _prebaseline_patch_number(*, target_commit: str, baseline_commit: str) -> int | None:
    # Deterministic ordering for pre-baseline commits.
    ordered = _run_git("rev-list", "--reverse", baseline_commit).splitlines()
    patch = 0
    for commit in ordered:
        if commit == baseline_commit:
            break
        patch += 1
        if commit == target_commit:
            return patch
    return None


def resolve(commit_ref: str) -> str:
    payload, releases = _validated_state()
    target_commit = _to_full_commit(commit_ref)
    baseline = payload["baseline"]
    baseline_commit = str(baseline["commit"])
    baseline_version = str(baseline["version"])

    commit_to_release_version = {release.commit: release.version_text for release in releases}
    exact_release = commit_to_release_version.get(target_commit)
    if exact_release is not None:
        return f"v{exact_release}"

    if target_commit == baseline_commit:
        return f"v{baseline_version}"

    if _is_ancestor(target_commit, baseline_commit):
        patch = _prebaseline_patch_number(target_commit=target_commit, baseline_commit=baseline_commit)
        if patch is None:
            raise VersionGuardError(
                f"Could not derive pre-baseline patch index for commit {target_commit}"
            )
        return f"v0.0.{patch}"

    nearest_release: Release | None = None
    for release in releases:
        if _is_ancestor(release.commit, target_commit):
            nearest_release = release

    if nearest_release is None:
        raise VersionGuardError(
            f"Commit {target_commit} is not in baseline ancestry and has no release ancestor"
        )

    distance_text = _run_git("rev-list", "--count", f"{nearest_release.commit}..{target_commit}")
    distance = int(distance_text)
    if nearest_release.version[0] == 0 and nearest_release.version[1] == 0:
        return f"v0.0.{nearest_release.version[2] + distance}"
    return f"v{nearest_release.version_text}-dev.{distance}+{target_commit[:7]}"


def baseline_commit() -> str:
    payload, _ = _validated_state()
    return str(payload["baseline"]["commit"])


def baseline_version() -> str:
    payload, _ = _validated_state()
    return f"v{payload['baseline']['version']}"


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Validate and resolve SemVer-to-commit mappings.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("validate", help="Validate docs/releases/version-map.json")

    resolve_parser = subparsers.add_parser("resolve", help="Resolve commit ref to version")
    resolve_parser.add_argument("--commit", default="HEAD", help="Commit ref to resolve (default: HEAD)")

    subparsers.add_parser("baseline-commit", help="Print baseline commit hash")
    subparsers.add_parser("baseline-version", help="Print baseline semantic version (v-prefixed)")

    return parser


def main() -> int:
    parser = _build_parser()
    args = parser.parse_args()

    try:
        if args.command == "validate":
            validate()
            return 0
        if args.command == "resolve":
            print(resolve(args.commit))
            return 0
        if args.command == "baseline-commit":
            print(baseline_commit())
            return 0
        if args.command == "baseline-version":
            print(baseline_version())
            return 0
    except VersionGuardError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    parser.print_help()
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
