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
      },
      animation: {
        "pulse-slow": "pulse 2.4s cubic-bezier(0.4, 0, 0.6, 1) infinite",
      },
    },
  },
  plugins: [],
};
