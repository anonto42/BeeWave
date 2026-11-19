# Design

## Context

See proposal.md - Why.
The Web Bluetooth client captures EAPOL frames and PMKIDs in `HandshakeState`. Currently, `buildPcap` in `web/src/shared/monitor/pcap.ts` generates a PCAP for a single `HandshakePair`. Similarly, `handshakeToHashcat22000` and `pmkidToHashcat22000` format single entries.
The client needs batch consolidation for both PCAP and Hashcat 22000 without affecting core/firmware code or wire protocol.

## Goals / Non-Goals

**Goals:**
- Provide `buildMultiPcap(pairs: readonly HandshakePair[]): Uint8Array<ArrayBuffer>` that collates all captured EAPOL frames across pairs, orders them chronologically, and emits a standard libpcap 2.4 file.
- Provide `exportAll22000(pairs: readonly HandshakePair[], pmkids: readonly PmkidRecord[], getEssid: (bssid: string) => string): string` to generate unified Hashcat 22000 output.
- Add `allPmkids(): PmkidRecord[]` on `HandshakeState`.
- Render batch export buttons in `features/handshake/` toolbar when crackable captures exist.
- Support 1-click clipboard copy of the full 22000 output.

**Non-Goals:**
- No changes to device firmware, protocol crates, or wire codec.
- No changes to individual pair export functionality.

## Decisions

1. **PCAP Packet Ordering by Timestamp (`at`)**:
   - Packets from all handshake pairs are flattened into a single list and sorted ascending by their millisecond capture timestamp `at`. This ensures Wireshark and `hcxpcapngtool` receive frames in chronological reception sequence.
   - Alternative considered: Grouping packets by pair. Rejected because real-world packet captures in libpcap format are strictly chronologically sequenced.

2. **Unified Hashcat 22000 Consolidation**:
   - `exportAll22000` evaluates each pair: if it contains M1/M3 + M2, it generates a `WPA*02*` line.
   - It iterates through all PMKIDs and generates a `WPA*01*` line for each.
   - Empty or null lines are skipped; the result is joined with `\n`.
   - Alternative considered: Separate batch export for PMKID vs Handshake. Rejected because Hashcat mode 22000 accepts both `WPA*01*` and `WPA*02*` in the same input file simultaneously.

3. **Dynamic Workspace Toolbar Actions**:
   - When any crackable handshake (M1+M2) or PMKID is present, the toolbar displays a batch action group with capture counts (`ALL .PCAP`, `ALL .22000`, `COPY ALL 22000`).
   - When no crackable handshakes or PMKIDs exist, the batch actions group is hidden to avoid clutter.

## Risks / Trade-offs

- [Large number of captured frames exceeding memory]: Captured pairs are already bounded to `CAPACITY = 32` in `HandshakeState`, ensuring total memory for PCAP assembly in browser RAM is well under a few hundred kilobytes.
