# Design

## Context

See `proposal.md` for motivation.
Currently, `web/src/features/handshake/` and `web/src/features/targets/` present isolated target information. Users cannot easily see if the radio is locked to the target's frequency or hopping across 13 channels. Furthermore, exports are disabled unless `p.complete` is true (all 4 messages), even though M1+M2 provides 100% of the cryptographic data required for Hashcat Mode 22000.

## Goals / Non-Goals

**Goals:**
- Provide a clear, unified Target Audit card with live step-by-step progress (`M1` through `M4`).
- Identify `M1+M2` as `CRACKABLE` and immediately enable `.22000`, `COPY`, and `.PCAP` actions.
- Provide a 1-click `[LOCK CH <N>]` / `[AUTO HOP]` channel focus button on target cards that immediately sends `{ type: "configureRadio", channel }`.
- Provide multi-target channel hopping advisory when targets span multiple channels.
- Display associated clients, probing clients (PNL), and deauth alerts directly inside the target card.

**Non-Goals:**
- No active frame injection or deauth transmission (strict receive-only invariant).
- No wire protocol changes (`crates/protocol` already supports `configureRadio` and `setHandshakeCapture`).

## Decisions

### Decision 1: Crackable Status Recognition
In WPA/WPA2, Message 1 carries the Authenticator Nonce (`ANonce`) and Message 2 carries the Supplicant Nonce (`SNonce`) and Message Integrity Code (`MIC`). Hashcat mode 22000 only requires M1+M2.
- State display:
  - `LISTENING FOR CLIENT`: Target capture active, no handshake packets heard yet.
  - `M1 SEEN · WAITING CLIENT`: AP transmitted ANonce; waiting for station response.
  - `CRACKABLE (M1+M2)`: M1 and M2 present; offline cracking is 100% possible.
  - `FULL 4-WAY (4/4)`: All 4 handshake messages captured.
  - `PMKID CAPTURED`: AP provided PMKID in M1; clientless cracking possible.
- Export enablement: Enable `.22000`, `COPY`, and `.PCAP` as soon as `(M1 || M3) && M2` or `PMKID` is present.

### Decision 2: Target Channel Pinning
Each target has a known channel from the scan table.
- A prominent button `[LOCK CH <N>]` allows the user to immediately pin the sniffer radio to that target's channel using `send({ type: "configureRadio", channel: target.channel })`.
- When pinned, the button displays `[LOCKED CH <N> ✓]`, and clicking it releases back to channel hopping (`channel: 0`).
- If multiple targets exist on different channels, an advisory note explains channel hopping and allows 1-click focus for any of them.

### Decision 3: Inline Target Intelligence
Each target card integrates:
- Connected client stations heard from 802.11 Data headers.
- Client stations searching for this target SSID (PNL).
- Real-time deauth alerts if someone is attacking this target.

## Risks / Trade-offs

- **[Risk]** Locking to one channel stops detecting beacons on other channels.
  → **Mitigation:** The UI clearly states when the radio is channel-locked vs hopping, and provides an instant `[AUTO HOP]` toggle to return to full-spectrum discovery.
