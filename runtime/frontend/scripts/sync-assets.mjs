import { cpSync, existsSync, lstatSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sourceDir = join(projectRoot, "assets");
const distDir = join(projectRoot, "dist");
const shouldSkipEntry = (name) => name.startsWith(".") || name.endsWith(".md");

const copyEntry = (sourcePath, targetPath) => {
  const stat = lstatSync(sourcePath);
  if (stat.isDirectory()) {
    mkdirSync(targetPath, { recursive: true });
    for (const entry of readdirSync(sourcePath, { withFileTypes: true })) {
      if (shouldSkipEntry(entry.name)) continue;
      copyEntry(join(sourcePath, entry.name), join(targetPath, entry.name));
    }
    return;
  }
  mkdirSync(dirname(targetPath), { recursive: true });
  cpSync(sourcePath, targetPath);
};

mkdirSync(distDir, { recursive: true });

if (!existsSync(sourceDir)) {
  console.log("[sync-assets] no assets directory found; skipping");
  process.exit(0);
}

const entries = readdirSync(sourceDir, { withFileTypes: true });
rmSync(join(distDir, "README.md"), { force: true });
for (const entry of entries) {
  if (shouldSkipEntry(entry.name)) continue;
  const sourcePath = join(sourceDir, entry.name);
  const targetPath = join(distDir, entry.name);
  rmSync(targetPath, { recursive: true, force: true });
  copyEntry(sourcePath, targetPath);
}

console.log("[sync-assets] copied assets to dist");
