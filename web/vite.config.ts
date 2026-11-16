import { defineConfig } from "vitest/config";
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig({
  base: "./",
  build: { target: "es2022", assetsInlineLimit: 0 },
  plugins: [
    VitePWA({
      registerType: "autoUpdate",
      workbox: { globPatterns: ["**/*.{js,css,html,svg}"] },
      manifest: {
        name: "WalPad",
        short_name: "WalPad",
        display: "standalone",
        background_color: "#2B3035",
        theme_color: "#2B3035",
        icons: [{ src: "icon.svg", sizes: "any", type: "image/svg+xml", purpose: "any" }],
      },
    }),
  ],
  test: { include: ["test/**/*.test.ts"] },
});
