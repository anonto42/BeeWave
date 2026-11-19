# Proposal

## Why

Security auditors and penetration testers using BeeWave frequently audit multiple access points and capture numerous WPA/WPA2 4-way handshakes and PMKIDs during a single monitoring session. Currently, the PWA requires exporting each handshake or PMKID individually as separate single-pair `.pcap` and `.22000` files. For workflows with cracking clusters or offline analysis tools (`hashcat`, `hcxtools`, `aircrack-ng`, `Wireshark`), downloading and feeding dozens of individual files is tedious and prone to omissions. Batch-consolidating all captured handshakes into a single chronological `.pcap` file and all hashes into a unified `.22000` file streamlines offline cracking and auditing.

## What Changes

- Add `buildMultiPcap(pairs: readonly HandshakePair[]): Uint8Array<ArrayBuffer>` to stitch all captured EAPOL messages across all target APs and client stations into a single standard libpcap 2.4 file sorted chronologically.
- Add `allPmkids(): PmkidRecord[]` to `HandshakeState` to retrieve all captured PMKIDs across all monitored access points.
- Add `exportAll22000(pairs: readonly HandshakePair[], pmkids: readonly PmkidRecord[], getEssid: (bssid: string) => string): string` to produce a consolidated multi-line Hashcat mode 22000 file combining all crackable handshakes (`WPA*02*`) and PMKIDs (`WPA*01*`).
- Add batch export controls to the Target Audit Workspace toolbar:
  - `ALL .PCAP`: Downloads `beewave-all-handshakes-<timestamp>.pcap` containing every captured EAPOL frame.
  - `ALL .22000`: Downloads `beewave-all-hashes-<timestamp>.22000` containing all valid handshake and PMKID hashes.
  - `COPY ALL 22000`: Copies the entire consolidated 22000 hash text to the clipboard with visual confirmation feedback.
  - Display badge indicating the total count of ready-to-crack captures.

## Capabilities

### Modified Capabilities

- `web-client`: Add requirement for batch PCAP and consolidated Hashcat mode 22000 multi-target export in the Target Audit Workspace.

## Impact

- Web client only (`web/src/shared/monitor/` and `web/src/features/handshake/`).
- Zero changes to wire format or firmware; strictly client-side aggregation of already captured telemetry.
