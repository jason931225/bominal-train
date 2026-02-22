import nextCoreWebVitals from "eslint-config-next/core-web-vitals";

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
    rules: {
      // Existing UI components intentionally set local state from effects for
      // route/theme synchronization and mount-time portal readiness.
      "react-hooks/set-state-in-effect": "off",
    },
  },
];

export default config;
