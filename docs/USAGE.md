# Using WalPad end to end

## What happens when you power the ESP32
1. It boots, starts Wi-Fi in station mode (never connects to any network), and applies your saved settings
   (channel, scan-at-boot).
2. It starts **BLE advertising** under the name `WalPad`. It keeps advertising until a phone connects.
3. If "scan at boot" is on, it starts sniffing beacons immediately; results only travel once a phone is connected.

## What the phone does
1. Open the PWA (hosted over HTTPS, see `docs/DEPLOYMENT.md`) in **Chrome/Edge on Android** (or desktop).
   iPhone: use the **Bluefy** browser. Safari cannot do Web Bluetooth.
2. Tap **CONNECT**. Chrome shows a chooser listing devices named `WalPad`. Pick it.
3. First time only: Android asks to **pair** and shows a passkey box. Type the 6-digit passkey
   (default `123456`; set your own with `WALPAD_PASSKEY=<6 digits> make fw-build`). The phone bonds with the device,
   so later connections skip this step.
4. The status panel starts updating once per second (channel, free heap, CPU %, packets dropped on the device,
   packets lost on the BLE link).

## Operations
| You do | What happens |
|---|---|
| START SCAN / STOP | Device sniffs beacons and hops channels 1-13 (or stays on a pinned channel). New or changed networks appear in the SCANNER table. |
| Pick a channel + SET CHANNEL | Pin the radio to that channel (AUTO = hop). |
| Tap a network row | Adds it to the TARGETS tray (saved in your browser). |
| SETTINGS -> SAVE TO DEVICE | Channel and scan-at-boot are written to flash and survive power loss. |
| FIRMWARE UPDATE | Choose `dist/walpad-app.bin`, the phone sends it over BLE in 96-byte chunks with acks, the device verifies SHA-256, switches slot and reboots. If the new image fails to boot, the bootloader rolls back. |
| REBOOT | Restarts the device. |

Make the update file with `make fw-image` (it also writes the SHA-256 for your own check).

## Limits to know
- One phone at a time (one BLE connection).
- Wi-Fi scanning and BLE share one radio, so notifications can be delayed while hopping; the BLE LOST counter shows real loss.
- Scanning only lists beacons (management frames); it does not capture traffic or connect to networks.
- Only use it on networks you own or are authorised to test.
