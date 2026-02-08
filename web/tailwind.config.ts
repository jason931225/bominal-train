import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./app/**/*.{js,ts,jsx,tsx,mdx}",
    "./components/**/*.{js,ts,jsx,tsx,mdx}",
    "./lib/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "var(--font-sans)",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "Segoe UI",
          "sans-serif",
        ],
        display: [
          "var(--font-display)",
          "ui-serif",
          "Georgia",
          "Cambria",
          "Times New Roman",
          "Times",
          "serif",
        ],
      },
      colors: {
        blossom: {
          50: "rgb(var(--blossom-50) / <alpha-value>)",
          100: "rgb(var(--blossom-100) / <alpha-value>)",
          200: "rgb(var(--blossom-200) / <alpha-value>)",
          300: "rgb(var(--blossom-300) / <alpha-value>)",
          400: "rgb(var(--blossom-400) / <alpha-value>)",
          500: "rgb(var(--blossom-500) / <alpha-value>)",
          600: "rgb(var(--blossom-600) / <alpha-value>)",
          700: "rgb(var(--blossom-700) / <alpha-value>)",
          800: "rgb(var(--blossom-800) / <alpha-value>)",
          900: "rgb(var(--blossom-900) / <alpha-value>)",
        },
      },
      boxShadow: {
        petal: "0 10px 28px -18px rgb(var(--shadow-petal) / 0.55)",
      },
    },
  },
  plugins: [],
};

export default config;
