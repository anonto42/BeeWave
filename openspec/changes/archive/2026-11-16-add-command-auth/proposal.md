## Why

Pairing is disabled (see `ble-link`), so anyone in Bluetooth range can start scans, reboot the
device or push firmware. Android's pairing screen did not bond reliably with this custom GATT
peripheral and Web Bluetooth cannot drive it, so bonding is not a dependable fix.

## What Changes

- Application-level authentication: the phone proves it knows a 32-byte shared key with an
  HMAC-SHA256 challenge-response before the device accepts any command.
- The key is set at build time (`BEEWAVE_KEY`, 64 hex chars) and entered once in the PWA.
  If no valid key is built in, the device stays locked (fails closed).
- Until authenticated, the device rejects every command except the auth handshake and sends no
  status or scan telemetry. The session ends on every connect and disconnect.
- Wire protocol: append `AuthBegin`, `AuthProve` commands and `AuthChallenge`, `AuthResult`
  telemetry (append-only, protocol version stays 1).

## Non-goals

- Encrypting the link (traffic is still readable by a sniffer; only command access is protected).
- BLE bonding, key rotation over the air, signed OTA images (separate changes).

## Impact

- `crates/protocol`, new `crates/core` auth slice, `App`, `System` port (random bytes),
  firmware BLE adapter + runtime event, web protocol codec and a new auth feature.
- RAM: about 100 bytes of state. Flash: sha2/hmac are already linked for OTA hashing.
