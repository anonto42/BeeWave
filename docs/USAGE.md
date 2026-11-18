<img src="../assets/logo/crest-primary.svg" alt="BeeWave logo" width="72" align="right">

# Using BeeWave end to end

## What happens when you power the ESP32
1. It boots, starts Wi-Fi in station mode (never connects to any network), and applies your saved settings
   (channel, scan-at-boot).
2. It starts **BLE advertising** under the name `BeeWave`. It keeps advertising until a phone connects.
3. If "scan at boot" is on, it starts sniffing beacons immediately; results only travel once a phone is connected.

## What the phone does
1. Open the PWA (hosted over HTTPS, see `docs/DEPLOYMENT.md`) in **Chrome/Edge on Android** (or desktop).
   iPhone: use the **Bluefy** browser. Safari cannot do Web Bluetooth.
2. The app has six tabs along the bottom: **Device**, **Scan**, **Analyze**, **Alerts**, **Settings** and **Log**. Tap **CONNECT** in the top bar (it
   also shows the link and key state: `connected`, `connected, no key`, `authenticating...`, `wrong key`). Chrome shows a chooser listing devices named `BeeWave`. Pick it.
3. First connect: nothing to enter. A fresh device is *unclaimed*; the PWA generates a random key, sends it once and
   saves it in this browser (the top bar goes to `authenticating...`, then `connected`). The device keeps the key in flash.
   A claimed device only accepts that key: another browser must paste it into **Settings > Connection > Device key** (use COPY KEY in the first browser), and
   `make fw-unclaim` resets the device so the next phone can claim it. Firmware built with `FW_KEY=file` has the key from
   `make fw-key` baked in instead.
4. After connecting, the phone proves it knows the key automatically; the panel shows `authenticated`.
   The status panel then updates once per second (channel, free heap, CPU %, packets dropped on the
   device, packets lost on the BLE link).

> **Security:** the device accepts no command, and sends no status or scan data, until the phone proves it
> knows the key (HMAC challenge-response, see `openspec/specs/ble-link/spec.md`). The Bluetooth link itself is
> **not encrypted**, so someone nearby can still read traffic; they just cannot control the device. A firmware
> that already has a key accepts only that key. **An unclaimed device can be claimed by anyone in range**, so connect
> right after flashing. The key lives in the device's flash and in your browser; keep both private.

## Operations
| You do | What happens |
|---|---|
| START SCAN / STOP SCAN (Scan tab) | One button: it starts a scan while idle and stops it while scanning; the CONTROL panel shows IDLE or SCANNING. Device sniffs beacons and hops channels 1-13 (or stays on a pinned channel). New or changed networks appear in the SCANNER table. |
| RADIO CHANNEL (Scan tab) | Picking a channel pins the radio to it at once (AUTO = hop). |
| SORT / CLEAR (Scan tab) | Sort the scanner table by signal, channel or name; CLEAR empties it (the device keeps scanning). |
| Tap a network | Marks it as a target (saved in your browser); TARGETS lists them with a REMOVE button. Signal is shown as bars plus dBm. |
| Settings > Scan / Radio / Telemetry / Alerts, then APPLY TO DEVICE | Device-side settings are validated, written to flash together and survive power loss. See `docs/SETTINGS.md`. |
| FIRMWARE UPDATE | Choose `dist/beewave-app.bin` and its `dist/beewave-app.bin.sig`. The phone sends the image over BLE in 96-byte chunks with acks; the device verifies the SHA-256 **and the Ed25519 signature**, switches slot and reboots. A missing or wrong signature is refused. If the new image fails to boot, the bootloader rolls back. |
| REBOOT DEVICE (Settings tab) | Restarts the device after a second tap on **CONFIRM REBOOT** (it disarms itself after 5 s). |
| Device error | Radio, update, settings and "locked" errors also pop up as a tap-to-dismiss alert above the tab bar (and stay in the Log tab). |

Make the update files with `make fw-image`: it writes the image, its `.sig` (signed with the private key in
`firmware/.beewave-ota.pem`, created once by `make fw-ota-key`, never leaves your computer) and the SHA-256. The matching
public key is built into the firmware, so the **first** signed update needs one cable flash of a build that has it.
A device built without an OTA public key refuses all updates.

## Monitoring
The **Analyze** and **Alerts** tabs hold the results table, channel chart, RSSI history, scan comparison, deauth detector, BLE monitor,
alert feed and watchlist; **Settings > Export session** saves the session as JSON or CSV. See `docs/MONITORING.md`.

## Limits to know
- One phone at a time (one BLE connection).
- Wi-Fi scanning and BLE share one radio, so notifications can be delayed while hopping; the BLE LOST counter shows real loss.
- Scanning only lists beacons (management frames); it does not capture traffic or connect to networks.
- Only use it on networks you own or are authorised to test.
