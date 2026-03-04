/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "../crates/api/src/**/*.rs",
    "../crates/shared/src/**/*.rs",
    "../crates/ui_patterns/src/**/*.rs",
    "../crates/ui_primitives/src/**/*.rs"
  ],
  theme: {
    extend: {
      colors: {
        bominal: {
          ink: "#0f172a",
          sky: "#0891b2",
          mint: "#059669",
          haze: "#f8fafc"
        }
      },
      keyframes: {
        "fade-slide": {
          "0%": { opacity: "0", transform: "translateY(10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" }
        }
      },
      animation: {
        "fade-slide": "fade-slide 480ms ease-out both"
      }
    }
  },
  plugins: []
};
