# Releases, Packaging and Deployments

This guide documents how BeeWave handles release versioning, firmware packaging, OTA signature generation, and Web PWA deployment.

---

## 1. Release Lifecycle & Versioning

BeeWave follows **Semantic Versioning** (`vMAJOR.MINOR.PATCH`):
- `MAJOR`: Incompatible wire-protocol changes or architecture overhauls.
- `MINOR`: New telemetry messages, attack detectors, scanner capabilities, or settings additions (backward-compatible).
- `PATCH`: Bug fixes, security hardening, or documentation improvements.

### Creating a Release Tag
```bash
# Ensure working tree is clean and all checks pass
make check

# Create an annotated git tag
git tag -a v0.1.0 -m "Release v0.1.0: Passive Wi-Fi & BLE security auditor"

# Push the tag to GitHub
git push origin v0.1.0
```

When a tag matching `v*.*.*` is pushed, the automated GitHub Action workflow (`.github/workflows/release.yml`) builds the production PWA bundle, computes SHA-256 checksums, and publishes a GitHub Release with downloadable tarballs.

---

## 2. Firmware Packaging & OTA Signing

Firmware binaries are compiled for the ESP32-WROOM target and signed with Ed25519 keys to prevent malicious firmware flashes over Web Bluetooth.

### Key Generation (One-Time)
Before building an OTA image, create the OTA signing key pair:
```bash
make fw-ota-key
```
- **Private key**: Stored in `firmware/.beewave-ota.pem` (untracked, never leaves your build machine).
- **Public key**: Stored in `firmware/.beewave-ota.pub` and compiled directly into the firmware binary as `BEEWAVE_OTA_PUBKEY`.

### Generating Signed Firmware Packages
Run:
```bash
make fw-image
```
This produces three release artifacts in `dist/`:
1. `beewave-app.bin`: Raw ESP32 application binary for OTA slot flashing.
2. `beewave-app.bin.sha256`: SHA-256 digest of the application image.
3. `beewave-app.bin.sig`: Ed25519 digital signature generated with your private key.

### Verifying the Key Fingerprint
Run `make fw-fingerprint` to print the 16-byte hex fingerprint of the public key:
```bash
make fw-fingerprint
```
In the Web PWA under **Settings > OTA**, the connected device reports its embedded fingerprint, allowing immediate verification that the uploaded image was signed by the legitimate key holder.

---

## 3. Web PWA Deployment

Because Web Bluetooth requires a **secure context** (HTTPS or `localhost`), the frontend PWA cannot be served directly from an ESP32 over HTTP.

### Production: GitHub Pages
- Workflow: `.github/workflows/pages.yml`
- Triggers: Automatically on pushes to `main` touching `web/**`.
- Mechanism: Installs dependencies via `pnpm`, executes Vitest tests, builds the PWA with Vite (`pnpm run build`), and deploys `web/dist/` to GitHub Pages.
- Installation: Users open the GitHub Pages URL on an Android device or desktop browser (Chrome/Edge) and select **"Install App"** or **"Add to Home screen"**. The service worker caches assets for 100% offline usage.

### Alternative Production Hosts
The static assets in `web/dist/` are self-contained and can be deployed to:
- **Cloudflare Pages**: Point build directory to `web/dist`.
- **Vercel / Netlify**: Configure root directory to `web/` and build command to `pnpm run build`.

### Local Development & Field Testing
1. **Local HTTPS Server (Same Wi-Fi)**:
   ```bash
   make web-lan
   ```
   Prints an `https://<ip>:5173` link accessible from mobile devices on the local network.
2. **USB Port Forwarding**:
   Use Chrome Remote Debugging (`chrome://inspect/#devices`) to forward `5173` over USB, allowing the phone to access `http://localhost:5173` as a trusted secure origin.
3. **Mock Mode (Hardware-Free)**:
   Append `?mock` to the URL or run `make web-mock` to test all scanner, analyzer, and alert views against simulated hardware.

---

## 4. Package Artifact Verification

Every release includes a `SHA256SUMS.txt` file. Verify downloaded packages before distribution:

```bash
sha256sum -c SHA256SUMS.txt
```
