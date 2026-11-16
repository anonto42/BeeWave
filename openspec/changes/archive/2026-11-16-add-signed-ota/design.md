## Signed message

The device signs/verifies the 32-byte SHA-256 of the whole image (the same digest `OtaBegin` already
carries and the flash adapter already checks while streaming). Order at `OtaEndSigned`:
1. all bytes received, 2. signature verified over the announced digest, 3. `OtaWriter::finish` re-checks
the streamed hash against that digest and marks the slot bootable. A failure at step 2 aborts the update.

## Wire

serde cannot encode `[u8; 64]`, so the signature is two `[u8; 32]` fields (R, S); on the wire that is
64 raw bytes, identical to the openssl signature file. `OtaEnd` stays in the enum (append-only) but is
always answered with an unsigned error.

## Keys

`DeviceKeys { auth, ota_signing }` is passed to `App::new`. Both come from compile-time env vars and
fail closed. Signing uses `openssl pkeyutl -sign -rawin` on the digest, so no custom tool is needed;
a host test checks an openssl-made signature against the Rust verifier.
