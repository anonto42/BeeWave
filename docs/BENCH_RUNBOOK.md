# Bench runbook

Do the steps in order and tick each one. Results go in `docs/ACCEPTANCE.md`; row IDs below are its rows.

**Before you start**
- Board on `/dev/ttyACM0`. Only one program can hold the serial port: use `make fw-run` (it flashes *and* monitors), or
  `make fw-monitor` alone, never both at once. Quit the monitor with Ctrl-C before flashing.
- The device has **no OLED or display**. Everything you can observe is the serial log and the PWA.
- The serial log does **not** print auth results or OTA results. It shows ESP-IDF boot lines, `beewave up on core N`,
  `BLE connected: ...`, `BLE disconnected: ...`, key errors (`... missing or not 64 hex characters; ...`) and a few warnings.
  Auth and OTA outcomes are visible only in the PWA (DEVICE KEY panel, LOG panel, FIRMWARE UPDATE status).
- **Known quirk (read before step 2):** on every connect the SETTINGS panel asks the device for its settings *before* the key
  handshake has finished, so the device refuses it. Expect **one extra `event 4: not authenticated` in the LOG right after each
  connect**, even with the right key, and the SETTINGS panel will **not** show the device's saved values after connecting. Ignore
  that line for S1 (S1 is about `authenticated` + status); where a step below counts `event 4` lines, count them *after* the connect.
  Nothing has been changed for this; tell me if you want it fixed.
- PWA texts you will see: DEVICE KEY panel = `no key saved` / `key saved, not connected` / `authenticating...` /
  `authenticated` / `rejected: wrong key?`. LOG lines look like `event 4: not authenticated`.
  STATUS cells: CHANNEL, FREE HEAP (in K), CPU %, DEV DROPPED, BLE LOST.
- Fill in the header of `docs/ACCEPTANCE.md` first (date, `make board-info`, `git rev-parse --short HEAD`, phone + browser).

## How to report a failure
Send: the row ID, the serial lines around the failure (from a `make fw-monitor` / `make fw-run` capture), and the exact
wrong behaviour (what you did, what you expected, what happened). If a step is unclear, ask before doing it.

---

## 1. Keys and first flash
- [ ] **Do:** `make fw-key`, then `make fw-ota-key`, then `make fw-run`.
- **Expect, serial:** ESP-IDF boot lines, then `beewave up on core <N>`. No panic, no reboot loop, and **no** line containing
  `LOCKED` or `DISABLED`.
- **Expect, PWA:** not opened yet.
- **Pass:** boots and stays up for about a minute with no panic or reboot loop (smoke test).
- **Row:** none (precondition for everything below).
- **If it fails, send:** the whole serial output from reset until the failure, and the output of `make fw-key` /
  `make fw-ota-key` / `make fw-run` (do **not** paste the key or the `.pem`).

## 2. PWA connect, right key
- [ ] **Do:** open the PWA in Chrome on the phone. Paste the output of `cat firmware/.beewave-key` into the DEVICE KEY
  field, tap **SAVE KEY**, tap **CONNECT**, pick `BeeWave`.
- **Expect, serial:** `BLE connected: ...`.
- **Expect, PWA:** connection state `connected`; DEVICE KEY goes `authenticating...` then `authenticated`; the STATUS cells fill
  in and update about once per second. (One stray `event 4: not authenticated` right at connect is the known quirk above.)
- **Pass (S1):** DEVICE KEY shows `authenticated`, status updates.
- **Row:** **S1**
- **If it fails, send:** S1, the DEVICE KEY text you saw, the LOG panel lines, and the serial lines from `BLE connected` onward.

## 3. Wrong key
- [ ] **Do:** run `openssl rand -hex 32` and paste the result into DEVICE KEY, tap **SAVE KEY**. Tap **DISCONNECT**, then **CONNECT**.
  Then tap **START SCAN**.
- **Expect, serial:** `BLE connected: ...` / `BLE disconnected: ...` only.
- **Expect, PWA:** DEVICE KEY shows `rejected: wrong key?`; STATUS cells stay `-`; SCANNER stays empty; the LOG gets
  a new `event 4: not authenticated` each time you tap START SCAN (on top of the connect-time one).
- **Pass (S2):** `rejected`; no status or scan data; commands do nothing.
- **Row:** **S2**
- **If it fails, send:** S2, which of {rejected text, status data, scan rows, command effect} was wrong, the LOG lines, serial lines.

## 4. No key
- [ ] **Do:** tap **FORGET**. Tap **DISCONNECT**, then **CONNECT**. Tap **START SCAN**.
- **Expect, serial:** `BLE connected: ...`.
- **Expect, PWA:** DEVICE KEY shows `no key saved`; STATUS stays `-`; SCANNER stays empty; LOG gets a new `event 4: not authenticated` when you tap START SCAN.
- **Pass (S3):** nothing shown or accepted.
- **Row:** **S3**
- **If it fails, send:** S3, what appeared or happened, LOG lines, serial lines.

## 5. Reconnect (session ends on disconnect)
- [ ] **Do:** paste the right key again and **SAVE KEY**; connect and wait for `authenticated`. Tap **DISCONNECT**. Tap **FORGET**.
  Tap **CONNECT** again, then tap **START SCAN**. Finally paste the right key, **SAVE KEY**, **DISCONNECT**, **CONNECT**.
- **Expect, serial:** `BLE connected` / `BLE disconnected` pairs.
- **Expect, PWA:** the reconnect without a key does not become authenticated and START SCAN only adds
  `event 4: not authenticated`; with the key saved again it authenticates by itself.
- **Pass (S4):** commands refused until the handshake runs again after a reconnect.
- **Row:** **S4**
- **If it fails, send:** S4, the sequence of DEVICE KEY texts, LOG lines, serial lines.

## 6. Scan, pin, save, reboot (authenticated)
Make sure DEVICE KEY shows `authenticated` first.
- [ ] **Scan.** **Do:** START SCAN, watch SCANNER for a minute, then STOP. **Expect, PWA:** rows appear; a network reappears only
  when it is new or its RSSI moved by about 6 dB or more; no new rows after STOP. **Pass (2.7):** de-dup holds. **Row:** **2.7**
- [ ] **Pin and AUTO.** **Do:** START SCAN, choose a channel, tap **SET CHANNEL**; then choose AUTO and tap **SET CHANNEL**.
  **Expect, PWA:** STATUS CHANNEL stays on the pinned number; with AUTO it changes (hops every 250 ms, shown once per second).
  **Pass (2.8):** stays on the pinned channel; AUTO resumes hopping. **Row:** **2.8**
- [ ] **Save and power-cycle.** **Do:** in SETTINGS choose a channel and tick SCAN AT BOOT, tap **SAVE TO DEVICE**, unplug and replug
  the board, reconnect (authenticates by itself). **Expect, PWA:** judge persistence from STATUS, not from the SETTINGS panel (it does not
  reload after connecting, see the known quirk): STATUS CHANNEL sits on the saved channel (if you saved a fixed one) and, with
  SCAN AT BOOT, SCANNER fills without you tapping START SCAN. **Pass (2.9):** channel and scan-at-boot restored. **Row:** **2.9**
- [ ] **Reboot.** **Do:** tap **REBOOT**. **Expect, serial:** a fresh boot and `beewave up on core <N>`, `BLE disconnected: ...`.
  **Expect, PWA:** state `disconnected`; reconnect authenticates again by itself. **Pass (2.10):** restarts, link drops, re-authenticates.
  **Row:** **2.10**
- **If any fails, send:** the row ID, serial lines around it, exactly what the PWA showed versus the expected text above.

## 7. Signed OTA
Run `make fw-image` first (writes `dist/beewave-app.bin`, `.sha256` and `.bin.sig`). Copy both files to the phone.
- [ ] **Good signature.** **Do:** FIRMWARE UPDATE: choose `beewave-app.bin` and `beewave-app.bin.sig`, tap **UPDATE FIRMWARE**.
  **Expect, PWA:** progress to 100%, status `update written and verified, rebooting into the new firmware...`, then the device
  reboots and the state goes `disconnected`. **Expect, serial:** reboot, `beewave up on core <N>`. Reconnect: authenticates, status resumes.
  **Pass (S6, 2.5):** completes, new image boots. **Row:** **S6**, **2.5**
- [ ] **Bad signature.** **Do:** `cp dist/beewave-app.bin.sig dist/bad.sig && printf '\x00' | dd of=dist/bad.sig bs=1 seek=0 conv=notrunc`
  (copy `bad.sig` to the phone). Update with `beewave-app.bin` + `bad.sig`.
  **Expect, PWA:** the upload runs to 100% (the signature is checked at the end), then status `failed: Error: ota refused: bad signature`
  and LOG `event 2: ota refused: bad signature`; the device does **not** reboot. Note: "at once" in ACCEPTANCE means no timeout wait
  after the end, not before the transfer. **Pass (S7):** refused with "bad signature"; old firmware keeps running (status keeps updating).
  **Row:** **S7**
- **If it fails, send:** the row ID, the FIRMWARE UPDATE status text, LOG lines, serial lines from the start of the update.

## 8. Builds without keys (S8, S5)
These two need a build the Makefile will not make, so run the cargo command by hand (empty value = no key).
After both, restore the normal build with `make fw-run`.
- [ ] **No OTA key (S8).** **Do:** `. ~/export-esp.sh && cd firmware && BEEWAVE_KEY=$(cat .beewave-key) BEEWAVE_OTA_PUBKEY= cargo espflash flash --release --port /dev/ttyACM0 --monitor`.
  Connect and authenticate, then try the update from step 7 (good `.sig`). **Expect, serial:** a line
  `BEEWAVE_OTA_PUBKEY missing or not 64 hex characters; firmware updates are DISABLED`. **Expect, PWA:** authenticates fine; the update
  fails immediately with `ota refused: unsigned or no signing key`. **Pass (S8):** any update is refused. **Row:** **S8**
- [ ] **Locked build (S5).** **Do:** `. ~/export-esp.sh && cd firmware && BEEWAVE_KEY= BEEWAVE_OTA_PUBKEY=$(cat .beewave-ota.pub) cargo espflash flash --release --port /dev/ttyACM0 --monitor`.
  Save the right key in the PWA and connect. **Expect, serial:** `BEEWAVE_KEY missing or not 64 hex characters; device is LOCKED`.
  **Expect, PWA:** DEVICE KEY ends at `rejected: wrong key?` even with the right key; no status; commands do nothing.
  **Pass (S5):** nothing can authenticate. **Row:** **S5**
- [ ] **Restore.** `make fw-run`, confirm step 2 works again.
- **If it fails, send:** the row ID, the first serial lines after boot, the PWA texts.

## 9. Coexistence (rows 2.1, 2.2)
- [ ] **Do:** `make fw-run FEATURES=coex_debug`. Connect, authenticate, START SCAN with channel AUTO. Leave it for **10 minutes**
  (read FREE HEAP during the first 5).
- **Expect, serial:** every 10 s `coex queue: n=.. p50<=..us p99<=..us max=..us` and `coex notify: ...`; any
  `BLE disconnected: ...` line is a failure for 2.2.
- **Expect, PWA:** FREE HEAP in K, BLE LOST not growing much, the link stays `connected`.
- **Pass (2.1):** FREE HEAP above 80K for the first 5 minutes. **Pass (2.2):** 0 disconnects over 10 minutes, low BLE LOST.
  (Also from TESTING.md: queue p99 at or under 50000 us.)
- **Row:** **2.1**, **2.2**
- **If it fails, send:** the row ID, the `coex` lines and any `BLE disconnected` lines with their timing, the FREE HEAP and BLE LOST
  readings (start and end). `REQUEST_CONN_PARAMS` stays `false` until you decide after 2.2.

## 10. Performance rows
- [ ] **2.3 Both cores used.** **Do:** START SCAN on AUTO and watch CPU %; note `beewave up on core <N>` from boot.
  **Expect:** CPU % shows a number above idle while scanning (it is one average over both cores; the tasks are not pinned).
  **Pass:** load is reported and tasks are unpinned. **Row:** **2.3**
- [ ] **2.6 OTA rollback (optional, as in TESTING.md).** Needs a deliberately crashing image that you build yourself and sign, and
  it must crash **before** `esp_ota_mark_app_valid_cancel_rollback` in `main()` or rollback will not trigger (that call runs
  first thing at boot). **Do not do this step without asking me first.** **Pass:** returns to the previous image. **Row:** **2.6**
- **If either fails, send:** the row ID, serial lines, the exact wrong behaviour.

## 11. Pages (rows 2.4)
- [ ] **Do:** in the GitHub repo set Settings -> Pages -> Source = **GitHub Actions**; wait for the `deploy-pwa` workflow to finish;
  open the Pages URL on the phone over HTTPS, install the PWA (Add to Home screen). Then airplane mode, clear the cache, open
  the installed PWA.
- **Expect, PWA:** the shell renders with no network; with no device, state stays `disconnected`.
  There is no in-app timer: measure the load with Chrome DevTools (`chrome://inspect` on the PC, Performance) or by eye.
- **Pass (2.4):** shell renders < 100 ms after load.
- **Row:** **2.4**
- **If it fails, send:** the row ID, the Pages URL, the workflow run result, what the phone showed (a screenshot is fine).
