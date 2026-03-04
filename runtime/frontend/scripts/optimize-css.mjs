import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import process from "node:process";
import { browserslistToTargets, transform } from "lightningcss";

function readArg(flag, fallback) {
  const index = process.argv.indexOf(flag);
  if (index === -1 || index + 1 >= process.argv.length) {
    return fallback;
  }
  return process.argv[index + 1];
}

const inputPath = resolve(readArg("--input", "./dist/tailwind.raw.css"));
const outputPath = resolve(readArg("--output", "./dist/tailwind.css"));
const browserTargets = readArg(
  "--targets",
  ">= 0.5%, last 2 versions, Firefox ESR, not dead",
);
const deleteInput = process.argv.includes("--delete-input");

const source = await readFile(inputPath);
const targets = browserslistToTargets(
  browserTargets
    .split(",")
    .map((query) => query.trim())
    .filter(Boolean),
);

const result = transform({
  filename: inputPath,
  code: source,
  minify: true,
  sourceMap: false,
  targets,
});

await mkdir(dirname(outputPath), { recursive: true });
await writeFile(outputPath, result.code);
if (deleteInput) {
  await rm(inputPath, { force: true });
}

console.log(
  `[optimize-css] wrote ${outputPath} (${result.code.length.toLocaleString()} bytes)`,
);
