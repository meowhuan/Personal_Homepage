import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import UnoCSS from "@unocss/vite";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [vue(), UnoCSS()],
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        donate: resolve(__dirname, "donate.html")
      }
    }
  }
});
