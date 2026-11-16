## Handshake

1. App sends `AuthBegin`. Device replies `AuthChallenge { nonce: [u8; 16] }` (fresh, single use).
2. App sends `AuthProve { mac }` where `mac = HMAC-SHA256(key, "beewave-auth-v1" || nonce)`.
3. Device verifies in constant time, clears the nonce, replies `AuthResult { ok }`.

A captured proof is useless later because the nonce is single use and replaced per `AuthBegin`.

## Core

New `features/auth` slice (`AuthGate`) depends only on `ports` and `protocol`. `App` composes it:
unauthenticated commands emit `Event` code 4 and are dropped, and `emit` only lets auth
messages and that event through until the session is authenticated. `System` gains
`fill_random` so the core never touches hardware entropy directly.

## Firmware

`BEEWAVE_KEY` is read at compile time and parsed once at boot; missing or invalid = locked.
The BLE adapter sends `Event::LinkReset` on connect and disconnect, which clears the session
(and aborts any OTA in flight). Reset uses a blocking send so it cannot be lost to a full queue.
`make fw-key` creates an untracked random key file so builds just work.

## Web

`shared/auth` holds the key store (localStorage, failure tolerant), the HMAC (SubtleCrypto)
and the handshake. A new `features/auth` slice takes the key and shows the auth state.
The handshake runs automatically when the transport connects and a key is stored.

## Risks

- Link is not encrypted: an attacker who captures traffic sees telemetry, but cannot
  issue commands without the key.
- Key lives in the browser's localStorage and in firmware flash; protect both physically.
