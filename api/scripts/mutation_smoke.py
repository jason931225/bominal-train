#!/usr/bin/env python3
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import subprocess
import sys


@dataclass(frozen=True)
class MutantCase:
    name: str
    file_path: Path
    needle: str
    replacement: str
    test_cmd: list[str]


def _run_case(api_dir: Path, case: MutantCase) -> None:
    target_path = api_dir / case.file_path
    original = target_path.read_text(encoding="utf-8")
    if case.needle not in original:
        raise RuntimeError(f"Mutation needle not found for {case.name}: {case.needle}")

    mutated = original.replace(case.needle, case.replacement, 1)
    target_path.write_text(mutated, encoding="utf-8")

    try:
        result = subprocess.run(case.test_cmd, cwd=api_dir, check=False)
        if result.returncode == 0:
            raise RuntimeError(f"Mutant survived: {case.name}")
        print(f"OK: mutant killed -> {case.name}")
    finally:
        target_path.write_text(original, encoding="utf-8")


def main() -> int:
    api_dir = Path(__file__).resolve().parents[1]
    cases = [
        MutantCase(
            name="pan redaction bypass",
            file_path=Path("app/core/crypto/redaction.py"),
            needle="redacted = _PAN_CANDIDATE_RE.sub(lambda match: _mask_pan(match.group(0)), redacted)",
            replacement="redacted = redacted",
            test_cmd=[
                sys.executable,
                "-m",
                "pytest",
                "--override-ini=addopts=",
                "-q",
                "tests/test_crypto_redaction.py",
            ],
        ),
        MutantCase(
            name="envelope unknown-kek fail-open",
            file_path=Path("app/core/crypto/envelope.py"),
            needle="if key is None:",
            replacement="if False and key is None:",
            test_cmd=[
                sys.executable,
                "-m",
                "pytest",
                "--override-ini=addopts=",
                "-q",
                "tests/test_crypto_envelope.py",
            ],
        ),
    ]

    try:
        for case in cases:
            _run_case(api_dir, case)
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    print(f"OK: API mutation smoke gate passed ({len(cases)} mutants killed).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
