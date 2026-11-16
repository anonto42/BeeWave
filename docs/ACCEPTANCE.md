# Hardware acceptance record

Fill this in on the bench after `make fw-run` (steps for the quick smoke test are in `docs/TESTING.md`).
One copy per firmware build you sign off; commit it with the results.

| Field | Value |
|---|---|
| Date | |
| Board / serial (`make board-info`) | |
| Firmware git SHA (`git rev-parse --short HEAD`) | |
| Phone + browser | |
| Built with `BEEWAVE_KEY` and `BEEWAVE_OTA_PUBKEY` | yes / no |

## Security (do these first)
| # | Check | How | Pass criterion | Result |
|---|---|---|---|---|
| S1 | Right key authenticates | Save the key, connect | DEVICE KEY shows `authenticated`, status updates | |
| S2 | Wrong key | Save a different 64-hex key, connect | `rejected`; no status or scan data; commands do nothing | |
| S3 | No key saved | Press FORGET, connect | Nothing shown or accepted | |
| S4 | Session ends on disconnect | Authenticate, disconnect, reconnect with no key | Commands refused until the handshake runs again | |
| S5 | Locked build | Flash a build without `BEEWAVE_KEY` | Serial log says `LOCKED`; nothing authenticates | |
| S6 | Signed OTA | `make fw-image`, update with `.bin` + `.sig` | Completes, new image boots | |
| S7 | Bad signature | Use a `.sig` from another build | Upload reaches 100%, then it is refused with "bad signature" (the signature is checked after the full image, before boot); old firmware keeps running | |
| S8 | No OTA key | Flash a build without `BEEWAVE_OTA_PUBKEY` | Any update is refused | |

## Performance and behaviour
| # | Check | Tool | Pass criterion | Result |
|---|---|---|---|---|
| 2.1 | Free heap, BLE connected + scanning | Status FREE HEAP for 5 min | > 80 KB throughout | |
| 2.2 | No BLE disconnects while hopping | Scan on AUTO for 10 min, read BLE LOST | 0 disconnects, low loss | |
| 2.3 | Both cores used | Status CPU, boot log `core N` | Load reported; tasks unpinned | |
| 2.4 | PWA cold start | Installed PWA, airplane mode, cleared cache | Shell renders < 100 ms after load | |
| 2.5 | OTA works | FIRMWARE UPDATE panel | New image boots, status resumes | |
| 2.6 | OTA rollback | Update with a deliberately crashing (signed) image | Device returns to the previous image | |
| 2.7 | Scan de-dup | Watch SCANNER | A network reappears only when new or RSSI moved >= 6 dB | |
| 2.8 | Channel pin | SET CHANNEL, then AUTO | Stays on the pinned channel; AUTO resumes hopping | |
| 2.9 | Settings persistence | SAVE TO DEVICE, power-cycle | Channel and scan-at-boot restored | |
| 2.10 | Reboot command | REBOOT | Device restarts, link drops, re-authenticates on reconnect | |

Notes / anomalies:
