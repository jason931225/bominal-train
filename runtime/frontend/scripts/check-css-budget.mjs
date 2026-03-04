import { readFileSync } from "node:fs";
import process from "node:process";
import zlib from "node:zlib";

function readArg(flag, fallback) {
  const index = process.argv.indexOf(flag);
  if (index === -1 || index + 1 >= process.argv.length) {
    return fallback;
  }
  return process.argv[index + 1];
}

function readNumberArg(flag, fallback) {
  const raw = readArg(flag, `${fallback}`);
  const parsed = Number.parseInt(raw, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`invalid value for ${flag}: ${raw}`);
  }
  return parsed;
}

const cssFile = readArg("--file", "./dist/tailwind.css");
const maxBytes = readNumberArg("--max-bytes", 70_000);
const maxGzipBytes = readNumberArg("--max-gzip-bytes", 12_000);

const css = readFileSync(cssFile);
const cssBytes = css.length;
const gzipBytes = zlib.gzipSync(css, { level: 9 }).length;

const failures = [];
if (cssBytes > maxBytes) {
  failures.push(`css bytes ${cssBytes} exceed budget ${maxBytes}`);
}
if (gzipBytes > maxGzipBytes) {
  failures.push(`gzip bytes ${gzipBytes} exceed budget ${maxGzipBytes}`);
}

if (failures.length > 0) {
  console.error(`[check-css-budget] failed for ${cssFile}`);
  failures.forEach((line) => console.error(`- ${line}`));
  process.exit(1);
}

console.log(
  `[check-css-budget] ok ${cssFile} (css=${cssBytes}, gzip=${gzipBytes})`,
);
