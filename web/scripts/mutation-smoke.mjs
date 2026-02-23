#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();

const cases = [
  {
    name: "dummy-task validation mutant",
    file: "lib/train/dummy-task-cards.ts",
    needle: "typeof value.id === \"string\" &&",
    replacement: "true &&",
    tests: ["lib/train/dummy-task-cards.test.ts"],
  },
  {
    name: "train provider badge mutant",
    file: "components/module-tile.tsx",
    needle: "if (module.slug === \"train\") {",
    replacement: "if (false && module.slug === \"train\") {",
    tests: ["components/__tests__/module-tile.providers.test.tsx"],
  },
];

function runVitest(testFiles) {
  const vitestBin = path.join(root, "node_modules", ".bin", "vitest");
  const args = ["run", "--reporter=dot", ...testFiles];
  const result = spawnSync(vitestBin, args, {
    cwd: root,
    stdio: "inherit",
    env: process.env,
  });
  return result.status ?? 1;
}

function mutateAndCheck(mutant) {
  const filePath = path.join(root, mutant.file);
  const original = fs.readFileSync(filePath, "utf8");
  if (!original.includes(mutant.needle)) {
    throw new Error(`Mutation needle not found in ${mutant.file}: ${mutant.needle}`);
  }
  const mutated = original.replace(mutant.needle, mutant.replacement);
  fs.writeFileSync(filePath, mutated, "utf8");
  try {
    const exitCode = runVitest(mutant.tests);
    if (exitCode === 0) {
      throw new Error(`Mutant survived: ${mutant.name}`);
    }
    console.log(`OK: mutant killed -> ${mutant.name}`);
  } finally {
    fs.writeFileSync(filePath, original, "utf8");
  }
}

function main() {
  for (const mutant of cases) {
    mutateAndCheck(mutant);
  }
  console.log(`OK: mutation smoke gate passed (${cases.length} mutants killed).`);
}

main();
