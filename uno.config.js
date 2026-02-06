import { defineConfig, presetUno, presetWebFonts } from "unocss";

export default defineConfig({
  presets: [
    presetUno(),
    presetWebFonts({
      provider: "google",
      fonts: {
        body: "M PLUS Rounded 1c",
        display: "ZCOOL XiaoWei"
      }
    })
  ],
  theme: {
    colors: {
      meow: {
        bg: "#F8F3F8",
        ink: "#2B1D2A",
        soft: "#7E6A86",
        accent: "#FF7AB6",
        mint: "#6ED6C2",
        card: "#FFF8FF",
        line: "#E9D9EA",
        night: {
          bg: "#1a1a2e",
          ink: "#f3e9ff",
          soft: "#b8a6d8",
          accent: "#88f3ff",
          mint: "#7fffd4",
          card: "#241f3d",
          line: "#332b55"
        }
      }
    },
    fontFamily: {
      body: ["M PLUS Rounded 1c", "ui-sans-serif", "system-ui"],
      display: ["ZCOOL XiaoWei", "ui-serif", "serif"]
    },
    keyframes: {
      floaty: {
        "0%, 100%": { transform: "translateY(0px) rotate(0deg)" },
        "50%": { transform: "translateY(16px) rotate(3deg)" }
      }
    },
    animation: {
      floaty: "floaty 10s ease-in-out infinite"
    }
  },
  shortcuts: {
    "meow-card": "rounded-2xl bg-meow-card/80 border border-meow-line shadow-[0_14px_30px_rgba(47,20,47,0.08)]",
    "meow-pill": "inline-flex items-center gap-2 rounded-full bg-white/70 px-3 py-1 text-xs text-meow-soft border border-meow-line",
    "meow-btn": "inline-flex items-center justify-center gap-2 rounded-full px-4 py-2 text-sm font-600 transition-transform duration-200",
    "meow-btn-primary": "meow-btn bg-meow-ink text-white hover:translate-y--0.5 hover:shadow-[0_0_0_3px_rgba(255,122,182,0.3)]",
    "meow-btn-ghost": "meow-btn border border-meow-line text-meow-ink hover:bg-white/70"
  },
  preflights: [
    {
      getCSS: () => `
        html, body, #app { min-height: 100%; }
        body { background: #F8F3F8; color: #2B1D2A; }
      `
    }
  ]
});
