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
      all: true,
      exclude: [
        "**/*.test.{ts,tsx}",
        "**/__tests__/**",
        "**/coverage/**",
        "messages/**",
        "**/*.d.ts",
      ],
      thresholds: {
        lines: 100,
        statements: 100,
        functions: 100,
        branches: 100,
      },
    },
  },
});
