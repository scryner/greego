/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: "#6366f1", // Indigo 500
        "background-light": "#f1f5f9", // Slate 100
        "background-dark": "#0f172a", // Slate 900
        "surface-light": "#ffffff",
        "surface-dark": "#1e293b", // Slate 800
        "border-light": "#e2e8f0",
        "border-dark": "#334155",
        "text-secondary-light": "#64748b",
        "text-secondary-dark": "#94a3b8",
      },
      fontFamily: {
        display: ["Inter", "sans-serif"],
      },
      borderRadius: {
        DEFAULT: "0.5rem",
        'xl': "1rem",
        '2xl': "1.5rem",
      },
    },
  },
  plugins: [],
}
