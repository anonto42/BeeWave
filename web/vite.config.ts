import { defineConfig } from "vitest/config";
import basicSsl from "@vitejs/plugin-basic-ssl";
import { VitePWA } from "vite-plugin-pwa";

export default defineConfig({
  base: "./",
  build: { target: "es2022", assetsInlineLimit: 0 },
  plugins: [
    // HTTPS=1 -> self-signed cert so a phone on the LAN gets a secure context (needed for Web Bluetooth).
    ...(process.env.HTTPS ? [basicSsl()] : []),
    VitePWA({
      registerType: "autoUpdate",
      workbox: { globPatterns: ["**/*.{js,css,html,svg}"] },
      manifest: {
        name: "BeeWave",
        short_name: "BeeWave",
        display: "standalone",
        background_color: "#2B3035",
        theme_color: "#2B3035",
        icons: [{ src: "icon.svg", sizes: "any", type: "image/svg+xml", purpose: "any" }],
      },
    }),
  ],
  test: { include: ["test/**/*.test.ts"] },
});
