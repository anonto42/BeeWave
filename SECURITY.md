# Security Policy

BeeWave takes the security and integrity of its firmware, communication channels, and users seriously.

## Supported Versions

Only the latest release and the current `main` branch receive security patches.

| Version | Supported |
|---|---|
| `main` (latest commit) | :white_check_mark: |
| Latest tagged release (`v0.1.x`) | :white_check_mark: |
| Older releases | :x: |

---

## Security Architecture & Design Guarantees

BeeWave is architected with defense-in-depth principles:

1. **Strictly Receive-Only**:
   BeeWave never transmits on the 802.11 physical layer: it does not send deauthentication frames, probe requests, probe responses, or beacon packets. The radio sniffer operates in passive promiscuous mode only.

2. **Application-Layer Command Authentication**:
   Because standard Web Bluetooth links may not provide end-to-end cryptographic link authentication, all device commands are gated behind an **HMAC-SHA256 challenge-response session**. A claimed device accepts no commands and emits no status/scan telemetry until the client proves possession of the 32-byte secret key stored in NVS.

3. **Ed25519-Signed Firmware Updates (OTA)**:
   Over-the-Air firmware updates require an Ed25519 digital signature verified against the embedded public key (`BEEWAVE_OTA_PUBKEY`) over the image SHA-256 before slot switching. Corrupted, tampered, or unsigned images are rejected, and ESP32 rollback partitions prevent bricking.

4. **Zero Heap Allocation in Core**:
   `crates/core` is `#![no_std]` and strictly uses bounded collections (`heapless`). It is memory-safe against dynamic allocation exhaustion and heap fragmentation attacks.

---

## Reporting a Vulnerability

If you discover a security vulnerability or security bug in BeeWave:

1. **Do not open a public GitHub issue.**
2. Send an email to the project maintainer:
   **MD Sohidul Islam Ananto** &lt;[anontom90@gmail.com](mailto:anontom90@gmail.com)&gt;
3. Please include:
   - A description of the vulnerability and its potential impact.
   - Exact steps or code to reproduce the issue.
   - Relevant hardware and software environment details (e.g., ESP32 revision, browser version, OS).

### Response Timeline
- **Initial Acknowledgement**: Within 48 hours.
- **Assessment & Status Update**: Within 7 days.
- **Coordinated Disclosure**: A patch will be prepared and tested prior to public release.
