#!/usr/bin/env python3
"""Fail-closed scanner for sensitive log leakage patterns."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

PAN_RE = re.compile(r"(?<!\d)(?:\d[ -]?){13,19}(?!\d)")
SENSITIVE_KEYWORDS = (
    "cvv",
    "authorization",
    "set-cookie",
    "cookie",
    "wrapped_dek",
    "ciphertext",
    "card_number",
)


def _luhn_ok(num: str) -> bool:
    total = 0
    digits = [int(ch) for ch in reversed(num)]
    for idx, value in enumerate(digits):
        if idx % 2 == 1:
            value *= 2
            if value > 9:
                value -= 9
        total += value
    return total % 10 == 0


def _line_has_pan(line: str) -> bool:
    for match in PAN_RE.finditer(line):
        digits = re.sub(r"\D", "", match.group(0))
        if 13 <= len(digits) <= 19 and _luhn_ok(digits):
            return True
    return False


def _line_has_unredacted_keyword(line: str) -> bool:
    lowered = line.lower()
    if "[redacted" in lowered:
        return False
    return any(keyword in lowered for keyword in SENSITIVE_KEYWORDS)


def _scan_stream(lines: list[str], source: str) -> list[str]:
    findings: list[str] = []
    for idx, line in enumerate(lines, start=1):
        if _line_has_pan(line):
            findings.append(f"{source}:{idx}: PAN-like value detected")
            continue
        if _line_has_unredacted_keyword(line):
            findings.append(f"{source}:{idx}: sensitive keyword without redaction marker")
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Scan logs for sensitive leakage patterns.")
    parser.add_argument("paths", nargs="*", help="Log files to scan. Reads stdin if omitted.")
    args = parser.parse_args()

    findings: list[str] = []

    if args.paths:
        for path_str in args.paths:
            path = Path(path_str)
            if not path.exists():
                findings.append(f"{path}: file not found")
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except Exception as exc:
                findings.append(f"{path}: read failed ({type(exc).__name__})")
                continue
            findings.extend(_scan_stream(text.splitlines(), str(path)))
    else:
        stdin_data = sys.stdin.read()
        findings.extend(_scan_stream(stdin_data.splitlines(), "stdin"))

    if findings:
        for finding in findings:
            print(finding)
        return 1

    print("scan_sensitive_logs: no sensitive patterns detected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
