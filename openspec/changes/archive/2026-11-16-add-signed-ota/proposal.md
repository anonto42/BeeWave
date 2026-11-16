## Why

Authenticated sessions stop strangers, but anyone who holds the shared key (or steals the phone's
stored key) could still push arbitrary firmware. A signature from a separate key that never leaves
the developer's computer means a leaked session key cannot install foreign code.

## What Changes

- Every firmware update must carry an Ed25519 signature over the image SHA-256, verified in the core
  before the flash writer may finish and mark the slot bootable.
- The public key is built into the firmware (`BEEWAVE_OTA_PUBKEY`); the private key stays in an
  untracked file created by `make fw-ota-key`. `make fw-image` writes `dist/beewave-app.bin.sig`.
- A device without a valid public key refuses all updates (fails closed). Flashing by cable still works.
- Wire protocol: append `OtaEndSigned { sig_r, sig_s }`. The plain `OtaEnd` is no longer accepted.
- The web client sends the `.sig` file and now waits for the device's real confirmation.

## Non-goals

- Secure boot or flash encryption, key rotation, anti-rollback versioning.

## Impact

`crates/protocol`, `crates/core` (OTA slice, `DeviceKeys`), firmware config, Makefile, web OTA feature.
Flash: ed25519-dalek adds roughly 20-30 KB. RAM: negligible.
