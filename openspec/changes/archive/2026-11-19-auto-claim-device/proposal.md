## Why

Entering the device key by hand on every new phone or browser is the main friction in using BeeWave
(an empty Status panel on first use was a locked device, not a fault). The owner asked for the device
and PWA to set themselves up automatically after the first connect.

## What Changes

- **Claim on first connect (trust on first use).** A device with no key is *unclaimed*. The first
  client that connects generates a random 32-byte key in the browser and sends it once (`Provision`).
  The device stores it in NVS flash and from then on is locked to that key (HMAC handshake as today).
  The PWA saves the same key automatically; nothing is typed or pasted.
- A key baked in at build time (`BEEWAVE_KEY`) still works and still wins: such a device is claimed
  from the start. `make fw-*` now builds **unclaimed** firmware by default; `FW_KEY=file` keeps the
  old behaviour.
- `make fw-unclaim` erases the stored key (NVS) so the owner can reset a claimed device.
- Wire protocol: append command `Provision { key }` and telemetry `Unclaimed` (append-only, version stays 1).

## Non-goals

- Encrypting the link, bonding, key rotation, multiple users per device (one key per device).
- Physical-presence pairing (button/QR). Evaluated and declined by the owner in favour of full automation.

## Risk (accepted by the owner)

Until the first claim, anyone in Bluetooth range can claim the device. After a claim, a stranger can
no longer take it over. This replaces the earlier invariant "no key = locked".

## Impact

`crates/protocol`, `crates/core` (auth gate, `App`, `SettingsStore` port), `firmware` (NVS key blob,
config, Makefile), `web` (authenticator, mock). RAM: 32 bytes. Flash: one NVS blob.
