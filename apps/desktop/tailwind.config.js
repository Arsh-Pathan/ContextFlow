/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {
      colors: {
        // ContextFlow brand surface — used by the floating bubble.
        bubble: {
          bg: "rgba(20, 22, 28, 0.78)",
          border: "rgba(255, 255, 255, 0.08)",
        },
        // Semantic theme tokens. Resolved at runtime from the active theme's
        // `--cf-*` custom properties (see src/theme/apply.ts). Layout code
        // references these names; switching theme only changes the values.
        cf: {
          bg: "var(--cf-bg)",
          elevated: "var(--cf-bg-elevated)",
          inset: "var(--cf-bg-inset)",
          text: "var(--cf-text)",
          muted: "var(--cf-text-muted)",
          faint: "var(--cf-text-faint)",
          border: "var(--cf-border)",
          "border-strong": "var(--cf-border-strong)",
          accent: "var(--cf-accent)",
          "accent-2": "var(--cf-accent-2)",
          "accent-contrast": "var(--cf-accent-contrast)",
        },
      },
      boxShadow: {
        "cf-glow": "0 0 24px rgba(var(--cf-glow-rgb), 0.45)",
        "cf-card": "0 1px 2px rgba(0,0,0,0.3), 0 8px 24px rgba(0,0,0,0.25)",
      },
      animation: {
        "pulse-slow": "pulse 2.4s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "cf-rise": "cf-rise 0.5s cubic-bezier(0.16, 1, 0.3, 1) both",
        "cf-fade-in": "cf-fade-in 0.3s ease both",
      },
      fontFamily: {
        // Offline-first: no webfonts (strict CSP). A curated native stack
        // gives the UI character without a network dependency.
        display: [
          '"Segoe UI Variable Display"',
          '"Segoe UI Variable"',
          '"Segoe UI"',
          "system-ui",
          "sans-serif",
        ],
        sans: [
          '"Segoe UI Variable Text"',
          '"Segoe UI Variable"',
          '"Segoe UI"',
          "system-ui",
          "sans-serif",
        ],
        mono: [
          '"Cascadia Code"',
          '"Cascadia Mono"',
          '"Consolas"',
          "ui-monospace",
          "monospace",
        ],
      },
    },
  },
  plugins: [],
};
