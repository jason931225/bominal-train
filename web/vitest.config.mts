import path from "node:path";
import { fileURLToPath } from "node:url";

import { configDefaults, defineConfig } from "vitest/config";

const rootDir = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  esbuild: {
    jsx: "automatic",
    jsxImportSource: "react",
  },
  resolve: {
    alias: {
      "@": path.resolve(rootDir, "."),
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./test/setup.ts"],
    exclude: [...configDefaults.exclude, "e2e/**", "playwright.config.ts"],
    clearMocks: true,
    restoreMocks: true,
    coverage: {
      provider: "v8",
      reporter: ["text", "json-summary", "lcov"],
      all: false,
      exclude: [
        "**/*.test.{ts,tsx}",
        "**/__tests__/**",
        "**/coverage/**",
        "messages/**",
        "**/*.d.ts",
      ],
      // Vitest 4 coverage accounting is stricter than v2 in this repo.
      // Rebaseline floors to the measured Vitest 4 baseline and ratchet upward over time.
      thresholds: {
        lines: 52,
        statements: 50,
        functions: 49,
        branches: 42,
      },
    },
  },
});
