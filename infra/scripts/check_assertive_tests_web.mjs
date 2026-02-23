#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";

let ts = null;
try {
  const requireFromWeb = createRequire(path.resolve(process.cwd(), "web/package.json"));
  ts = requireFromWeb("typescript");
} catch {
  // Fallback parser path is used when TypeScript is unavailable in current runtime.
}

function walk(dirPath) {
  const files = [];
  for (const entry of fs.readdirSync(dirPath, { withFileTypes: true })) {
    const fullPath = path.join(dirPath, entry.name);
    if (entry.isDirectory()) {
      files.push(...walk(fullPath));
      continue;
    }
    files.push(fullPath);
  }
  return files;
}

function callName(node) {
  if (ts.isIdentifier(node)) return node.text;
  if (ts.isPropertyAccessExpression(node)) return node.name.text;
  return null;
}

function isPrimitiveLiteral(node) {
  return (
    ts.isStringLiteralLike(node) ||
    ts.isNumericLiteral(node) ||
    node.kind === ts.SyntaxKind.TrueKeyword ||
    node.kind === ts.SyntaxKind.FalseKeyword ||
    node.kind === ts.SyntaxKind.NullKeyword
  );
}

function samePrimitiveLiteral(a, b) {
  if (!isPrimitiveLiteral(a) || !isPrimitiveLiteral(b)) return false;
  return a.getText() === b.getText();
}

function isExpectRootCall(call) {
  if (!ts.isCallExpression(call)) return false;
  if (ts.isIdentifier(call.expression) && call.expression.text === "expect") return true;
  if (
    ts.isPropertyAccessExpression(call.expression) &&
    ts.isIdentifier(call.expression.expression) &&
    call.expression.expression.text === "expect"
  ) {
    return true;
  }
  return false;
}

function findBaseExpectCall(expr) {
  let current = expr;
  while (ts.isPropertyAccessExpression(current) || ts.isElementAccessExpression(current)) {
    current = current.expression;
  }
  if (ts.isCallExpression(current) && isExpectRootCall(current)) {
    return current;
  }
  return null;
}

function expectMatcherState(call) {
  if (!ts.isCallExpression(call)) return { signal: false, meaningful: false };
  if (!ts.isPropertyAccessExpression(call.expression)) return { signal: false, meaningful: false };

  const matcher = call.expression.name.text;
  const expectCall = findBaseExpectCall(call.expression.expression);
  if (!expectCall) return { signal: false, meaningful: false };

  let vacuous = false;
  const expectArg = expectCall.arguments[0];
  const matcherArg = call.arguments[0];

  if (["toBe", "toEqual", "toStrictEqual"].includes(matcher) && expectArg && matcherArg) {
    vacuous = samePrimitiveLiteral(expectArg, matcherArg);
  }
  if (matcher === "toBeTruthy" && expectArg) {
    vacuous = expectArg.kind === ts.SyntaxKind.TrueKeyword;
  }
  if (matcher === "toBeFalsy" && expectArg) {
    vacuous = expectArg.kind === ts.SyntaxKind.FalseKeyword || expectArg.kind === ts.SyntaxKind.NullKeyword;
  }

  return { signal: true, meaningful: !vacuous };
}

function assertCallState(call) {
  const name = callName(call.expression);
  if (!name || !name.startsWith("assert")) return { signal: false, meaningful: false };
  if (name === "assert" && call.arguments[0] && call.arguments[0].kind === ts.SyntaxKind.TrueKeyword) {
    return { signal: true, meaningful: false };
  }
  return { signal: true, meaningful: true };
}

function callbackAssertionState(callback) {
  if (!callback) return { signal: false, meaningful: false };
  if (ts.isArrowFunction(callback) && !ts.isBlock(callback.body)) {
    if (!ts.isCallExpression(callback.body)) return { signal: false, meaningful: false };
    const matcherState = expectMatcherState(callback.body);
    if (matcherState.signal) return matcherState;
    return assertCallState(callback.body);
  }

  let sawSignal = false;
  let sawMeaningful = false;
  function visit(node) {
    if (sawMeaningful) return;
    if (ts.isCallExpression(node)) {
      const matcherState = expectMatcherState(node);
      if (matcherState.signal) {
        sawSignal = true;
        if (matcherState.meaningful) {
          sawMeaningful = true;
          return;
        }
      }

      if (isExpectRootCall(node)) {
        sawSignal = true;
      }

      const assertState = assertCallState(node);
      if (assertState.signal) {
        sawSignal = true;
        if (assertState.meaningful) {
          sawMeaningful = true;
          return;
        }
      }

      const directName = callName(node.expression);
      if (directName === "fail") {
        sawSignal = true;
        sawMeaningful = true;
        return;
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(callback);
  return { signal: sawSignal, meaningful: sawMeaningful };
}

function fallbackFindBodyRange(source, startIndex) {
  let i = startIndex;
  while (i < source.length && source[i] !== "{") i += 1;
  if (i >= source.length) return null;

  const bodyStart = i;
  let depth = 1;
  i += 1;
  while (i < source.length && depth > 0) {
    const ch = source[i];
    if (ch === "{") depth += 1;
    if (ch === "}") depth -= 1;
    i += 1;
  }
  if (depth !== 0) return null;
  return { start: bodyStart + 1, end: i - 1 };
}

function checkFileFallback(filePath, source) {
  const pattern = /\b(?:it|test)\s*\(\s*(?:"[^"]*"|'[^']*'|`[^`]*`)\s*,/g;
  const violations = [];
  let match;
  while ((match = pattern.exec(source)) !== null) {
    const bodyRange = fallbackFindBodyRange(source, pattern.lastIndex);
    if (!bodyRange) continue;
    const body = source.slice(bodyRange.start, bodyRange.end);
    const hasSignal = /\bexpect\s*\(|\bassert\w*\s*\(/.test(body);
    const hasVacuous = /\bexpect\s*\(\s*true\s*\)\s*\.toBe\s*\(\s*true\s*\)/.test(body);
    if (!hasSignal) {
      const line = source.slice(0, match.index).split("\n").length;
      violations.push({ filePath, line, reason: "missing assertion signal" });
    } else if (hasVacuous) {
      const line = source.slice(0, match.index).split("\n").length;
      violations.push({ filePath, line, reason: "vacuous assertion only" });
    }
    pattern.lastIndex = bodyRange.end + 1;
  }
  return violations;
}

function checkFile(filePath) {
  const source = fs.readFileSync(filePath, "utf8");
  if (!ts) {
    return checkFileFallback(filePath, source);
  }
  const sourceFile = ts.createSourceFile(filePath, source, ts.ScriptTarget.Latest, true);
  const violations = [];

  function visit(node) {
    if (ts.isCallExpression(node)) {
      const name = callName(node.expression);
      if ((name === "it" || name === "test") && node.arguments.length >= 2) {
        const callback = node.arguments[1];
        if (ts.isArrowFunction(callback) || ts.isFunctionExpression(callback)) {
          const state = callbackAssertionState(callback);
          const { line } = sourceFile.getLineAndCharacterOfPosition(node.getStart(sourceFile));
          if (!state.signal) {
            violations.push({ filePath, line: line + 1, reason: "missing assertion signal" });
          } else if (!state.meaningful) {
            violations.push({ filePath, line: line + 1, reason: "vacuous assertion only" });
          }
        }
      }
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return violations;
}

function main() {
  const root = process.argv[2] ?? "web";
  const candidates = walk(root).filter((filePath) => /\.test\.(ts|tsx)$/.test(filePath));
  const violations = [];
  for (const filePath of candidates) {
    violations.push(...checkFile(filePath));
  }

  if (violations.length === 0) {
    console.log(`OK: assertive-test check passed for ${candidates.length} web test files.`);
    process.exit(0);
  }

  console.error("ERROR: non-assertive web tests found:");
  for (const item of violations) {
    console.error(`  - ${item.filePath}:${item.line} (${item.reason})`);
  }
  process.exit(1);
}

main();
