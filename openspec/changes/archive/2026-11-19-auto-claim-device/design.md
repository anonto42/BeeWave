## Flow

```
connect -> AuthBegin
  device unclaimed -> Unclaimed          PWA: random key -> Provision{key}
                                         device: store key (NVS), claim gate, session authenticated -> AuthResult{ok:true}
                                         PWA: saves the key only after ok:true
  device claimed   -> AuthChallenge      PWA: HMAC with its saved key -> AuthProve -> AuthResult
                                         (PWA without a key: state "no-key", user pastes it or resets the device)
```

## Ports and failure modes

- `SettingsStore` gains `load_key() -> Option<[u8;32]>` and `save_key(&[u8;32]) -> Result<(), SettingsError>`.
  The key is persisted *before* the gate is claimed; a storage error leaves the device unclaimed and
  answers `AuthResult{ok:false}` plus a settings event.
- `AuthGate::claim(key)` succeeds only while the gate has no key; the all-zero key is refused.
  A `Provision` on a claimed device answers `AuthResult{ok:false}` and changes nothing.
- Key precedence at boot: build-time `BEEWAVE_KEY`, else NVS, else unclaimed.
- `Unclaimed` and the provisioning `AuthResult` are the only messages allowed before authentication
  (besides the existing handshake and unauthorized event).

## Security notes

The key is stored in plain NVS (flash), as the build-time key already was in the image: physical access
exposes it either way. The claim window is open from first boot until the first successful `Provision`.
