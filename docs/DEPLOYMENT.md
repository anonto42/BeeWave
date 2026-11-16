# Hosting the PWA

Web Bluetooth needs a **secure context** (HTTPS or `localhost`), so the PWA cannot be served from the ESP32 over HTTP.

## Production: GitHub Pages
`.github/workflows/pages.yml` runs the web tests, builds `web/` and deploys `web/dist` to GitHub Pages on every push to
`main` that touches `web/`. One-time setup in the GitHub repo: Settings -> Pages -> Source: **GitHub Actions**.
Open the Pages URL on the phone, then "Add to Home screen" to install the PWA; it then runs offline from the service worker.

## Testing on your phone before that
Pick one:
1. **LAN over HTTPS (simplest):** `make web-lan`, then open the printed `https://<your-pc-ip>:5173` on the phone
   (same Wi-Fi). Accept the self-signed certificate warning once. Chrome treats it as a secure context.
2. **USB port forwarding:** enable USB debugging on the phone, open `chrome://inspect/#devices` on the PC, add port
   `5173` under Port forwarding, run `make web-dev`, then open `http://localhost:5173` on the phone.
Use `?mock` at the end of the URL to try the UI with no device.
