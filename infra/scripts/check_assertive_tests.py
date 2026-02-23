#!/usr/bin/env python3
from __future__ import annotations

import argparse
import ast
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

_MISSING = object()


@dataclass(frozen=True)
class Violation:
    file_path: Path
    function_name: str
    line: int
    reason: str


def _iter_test_functions(tree: ast.AST) -> Iterable[ast.FunctionDef | ast.AsyncFunctionDef]:
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name.startswith("test_"):
            yield node


def _call_name(call: ast.Call) -> str | None:
    func = call.func
    if isinstance(func, ast.Name):
        return func.id
    if isinstance(func, ast.Attribute):
        return func.attr
    return None


def _is_pytest_raises_with_item(node: ast.With | ast.AsyncWith) -> bool:
    for item in node.items:
        ctx = item.context_expr
        if isinstance(ctx, ast.Call):
            name = _call_name(ctx)
            if name == "raises":
                return True
    return False


def _is_constant_expression(node: ast.AST) -> bool:
    if isinstance(node, ast.Constant):
        return True
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return all(_is_constant_expression(elt) for elt in node.elts)
    if isinstance(node, ast.Dict):
        return all(
            _is_constant_expression(key) and _is_constant_expression(value)
            for key, value in zip(node.keys, node.values)
            if key is not None
        )
    if isinstance(node, ast.UnaryOp):
        return _is_constant_expression(node.operand)
    if isinstance(node, ast.BoolOp):
        return all(_is_constant_expression(value) for value in node.values)
    if isinstance(node, ast.BinOp):
        return _is_constant_expression(node.left) and _is_constant_expression(node.right)
    if isinstance(node, ast.Compare):
        return _is_constant_expression(node.left) and all(
            _is_constant_expression(comp) for comp in node.comparators
        )
    return False


def _const_value(node: ast.AST) -> object:
    return node.value if isinstance(node, ast.Constant) else _MISSING


def _is_vacuous_assert_statement(node: ast.Assert) -> bool:
    return _is_constant_expression(node.test)


def _is_vacuous_assert_call(call: ast.Call, name: str) -> bool:
    args = list(call.args)
    if name == "assertTrue" and args and isinstance(args[0], ast.Constant):
        return bool(args[0].value) is True
    if name == "assertFalse" and args and isinstance(args[0], ast.Constant):
        return bool(args[0].value) is False
    if name in {"assertIsNone"} and args and isinstance(args[0], ast.Constant):
        return args[0].value is None
    if name in {"assertIsNotNone"} and args and isinstance(args[0], ast.Constant):
        return args[0].value is not None
    if name in {"assertEqual", "assertEquals", "assertIs"} and len(args) >= 2:
        left, right = _const_value(args[0]), _const_value(args[1])
        return left is not _MISSING and right is not _MISSING and left == right
    if name in {"assertNotEqual", "assertIsNot"} and len(args) >= 2:
        left, right = _const_value(args[0]), _const_value(args[1])
        return left is not _MISSING and right is not _MISSING and left != right
    return False


def _assertiveness_state(fn: ast.FunctionDef | ast.AsyncFunctionDef) -> tuple[bool, bool]:
    saw_signal = False
    saw_meaningful = False
    for node in ast.walk(fn):
        if isinstance(node, ast.Assert):
            saw_signal = True
            if not _is_vacuous_assert_statement(node):
                saw_meaningful = True
        if isinstance(node, (ast.With, ast.AsyncWith)) and _is_pytest_raises_with_item(node):
            saw_signal = True
            saw_meaningful = True
        if isinstance(node, ast.Call):
            name = _call_name(node)
            if not name:
                continue
            if name.startswith("assert"):
                saw_signal = True
                if not _is_vacuous_assert_call(node, name):
                    saw_meaningful = True
            if name in {"raises", "fail"}:
                saw_signal = True
                saw_meaningful = True
    return saw_signal, saw_meaningful


def _scan_file(path: Path) -> list[Violation]:
    try:
        source = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return []
    tree = ast.parse(source, filename=str(path))
    violations: list[Violation] = []
    for fn in _iter_test_functions(tree):
        saw_signal, saw_meaningful = _assertiveness_state(fn)
        if not saw_signal:
            violations.append(
                Violation(
                    file_path=path,
                    function_name=fn.name,
                    line=fn.lineno,
                    reason="missing assertion signal",
                )
            )
        elif not saw_meaningful:
            violations.append(
                Violation(
                    file_path=path,
                    function_name=fn.name,
                    line=fn.lineno,
                    reason="vacuous assertion only",
                )
            )
    return violations


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Fail when Python tests do not contain assertive checks.",
    )
    parser.add_argument(
        "paths",
        nargs="*",
        default=["api/tests"],
        help="Directories/files to scan (default: api/tests).",
    )
    args = parser.parse_args()

    test_files: list[Path] = []
    for raw in args.paths:
        path = Path(raw)
        if path.is_file():
            test_files.append(path)
            continue
        if not path.exists():
            continue
        test_files.extend(sorted(path.rglob("test_*.py")))

    violations: list[Violation] = []
    for file_path in test_files:
        violations.extend(_scan_file(file_path))

    if not violations:
        print(f"OK: assertive-test check passed for {len(test_files)} Python test files.")
        return 0

    print("ERROR: non-assertive Python tests found:")
    for item in violations:
        print(f"  - {item.file_path}:{item.line} {item.function_name} ({item.reason})")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
