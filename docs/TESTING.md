# Hardware test checklist

Run after `make fw-run`. Record results in the table at the bottom.

## Smoke test
- [ ] Serial log shows `beewave up` and no panic / reboot loop (`make fw-monitor`).
- [ ] Phone lists `BeeWave` in the Chrome chooser; pairing asks for the passkey; connect succeeds.
- [ ] Status panel updates every second.
- [ ] START SCAN lists nearby networks; STOP stops new rows.
- [ ] SET CHANNEL pins the channel shown in Status; AUTO makes it change.
- [ ] SAVE TO DEVICE, power-cycle the board, reconnect: settings are restored.
- [ ] Reboot from the UI, reconnect without re-pairing.

## Acceptance matrix (from the architecture plan)
| Test | How | Target | Result |
|---|---|---|---|
| Free heap with BLE connected + scanning | Status "FREE HEAP" after a minute | > 80 KB | |
| BLE loss while hopping | Scan on AUTO for 5 min, read "BLE LOST" | no disconnects, low loss | |
| Core usage | Run a scan; `core N` is logged at boot, tasks are unpinned | both cores used | |
| PWA cold start | Airplane mode, cleared cache, installed PWA | shell renders < 100 ms after load | |
| OTA | Update with `dist/beewave-app.bin` | completes, new image boots, status resumes | |
| OTA rollback | (optional) flash a deliberately crashing image via OTA | device returns to the previous image | |
