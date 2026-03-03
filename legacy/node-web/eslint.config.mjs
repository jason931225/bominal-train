import nextCoreWebVitals from "eslint-config-next/core-web-vitals";
import vitest from "@vitest/eslint-plugin";

const config = [
  {
    ignores: [
      ".next/**",
      "coverage/**",
      "node_modules/**",
    ],
  },
  ...nextCoreWebVitals,
  {
    files: ["**/*.{test,spec}.{ts,tsx,js,jsx}", "**/__tests__/**/*.{ts,tsx,js,jsx}"],
    plugins: {
      vitest,
    },
    languageOptions: {
      globals: {
        ...vitest.environments.env.globals,
      },
    },
    rules: {
      ...vitest.configs.recommended.rules,
    },
  },
  {
    rules: {
      // Existing UI components intentionally set local state from effects for
      // route/theme synchronization and mount-time portal readiness.
      "react-hooks/set-state-in-effect": "off",
    },
  },
];

export default config;
