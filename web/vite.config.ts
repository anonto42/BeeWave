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
      workbox: { globPatterns: ["**/*.{js,css,html,svg,png,ico}"] },
      includeAssets: ["favicon.ico", "favicon.svg", "apple-touch-icon.png", "icon.svg"],
      manifest: {
        name: "BeeWave",
        short_name: "BeeWave",
        display: "standalone",
        background_color: "#2B3035",
        theme_color: "#2B3035",
        description: "Wi-Fi and Bluetooth environment monitor for the BeeWave ESP32.",
        icons: [
          { src: "icon.svg", sizes: "any", type: "image/svg+xml", purpose: "any" },
          { src: "icon-192.png", sizes: "192x192", type: "image/png", purpose: "any" },
          { src: "icon-512.png", sizes: "512x512", type: "image/png", purpose: "any" },
          { src: "icon-maskable-512.png", sizes: "512x512", type: "image/png", purpose: "maskable" },
        ],
      },
    }),
  ],
  test: { include: ["test/**/*.test.ts"] },
});
