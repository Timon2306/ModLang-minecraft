/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        mc: {
          bg: "#121214",
          surface: "#18181c",
          surfaceHover: "#222228",
          panel: "#202026",
          slot: "#101014",
          borderDark: "#0c0c0e",
          borderLight: "#3c3c46",
          btnGray: "#4e4e58",
          btnGrayHover: "#5e5e6a",
          btnGreen: "#2e7d32",
          btnGreenHover: "#388e3c",
          btnRed: "#c62828",
          btnRedHover: "#d32f2f",
          btnGold: "#b78103",
          btnGoldHover: "#c78d04",
          text: "#e0e0e0",
          textMuted: "#8e8e98",
          emerald: "#55FF55",
          redstone: "#FF5555",
          gold: "#FFAA00",
          diamond: "#55FFFF",
          lapis: "#5555FF",
          purple: "#AA00AA",
        },
      },
      fontFamily: {
        pixel: ["Minecraftia", "VT323", "monospace"],
        mono: ["'JetBrains Mono'", "Consolas", "monospace"],
        sans: ["system-ui", "-apple-system", "BlinkMacSystemFont", "Segoe UI", "Roboto", "sans-serif"],
      },
    },
  },
  plugins: [],
};
