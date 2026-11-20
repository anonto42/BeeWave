# System Capabilities & Detectors

Canonical summary of domain capabilities implemented across `crates/protocol`, `crates/core`, `firmware/`, and `web/`:

## 1. Passive Wi-Fi Monitoring & Classification
- **Beacon Frame Parsing**: Zero-copy SSID, channel, and RSSI extraction (`crates/core/src/features/scan/frame.rs`).
- **PHY Generation Classification**: 802.11b, 802.11g, Wi-Fi 4 (802.11n Tag 45), and Wi-Fi 6 (802.11ax Tag 255 Ext 35) classification via `WifiPhyStandard` and `TelemetryMessage::PhyResult`.
- **WPS Fingerprinting**: WPS 1.0/2.0 discovery, AP setup lockout detection, Push-Button Configuration (PBC) state, and hardware manufacturer/model extraction (`WpsResult`).
- **Security & PMF Classification**: Protocol detection (`Open`, `WEP`, `WPA`, `WPA2`, `WPA3`, `OWE`) and Protected Management Frames (`PMF`) capability extraction (`SecurityResult`).
- **Hidden SSID Decloaking**: Correlation of client probe requests with hidden beacon BSSIDs to decloak hidden network names (`DecloakedResult`).

## 2. Attack Detection & Rogue AP Anomalies
- **Deauthentication Flood Detection**: Sliding-window rate detector alerting when deauth frames reach threshold (`DeauthAlert`).
- **Disassociation Flood Detection**: Dedicated disassociation attack detection and alerting (`DisassocAlert`).
- **Beacon Flood Detection**: Rogue AP SSID flooding detection based on frame rate and unique BSSID counts (`BeaconFloodAlert`).
- **Probe Request Flood Detection**: High-rate client probe request flood detection (`ProbeFloodAlert`).
- **Beacon Timestamp (TSF) Inversion**: Tracking 64-bit microsecond monotonic TSF progression to detect rogue AP timestamp rollbacks $\ge 5$ s (`AnomalyKind::TsfRollback`).
- **KARMA / Rogue AP Mutation**: Flagging APs broadcasting multiple distinct SSIDs on the same BSSID (`AnomalyKind::KarmaSsidChange`).
- **Evil Twin Security Downgrade**: Detecting security downgrade attacks against known network SSIDs (`AnomalyKind::SecurityDowngrade`).
- **Channel & Multi-BSSID Anomalies**: Flagging APs hopping channels within 60 s or unknown BSSIDs advertising known corporate SSIDs.

## 3. BLE Tracking & Coexistence
- **BLE Advertisement Classification**: Real-time stats and spike detection for Google Fast Pair, Apple iBeacon, Microsoft Proximity, and Generic advertisements (`BleStats`, `BleSpike`).
- **Apple AirTag / Find My Detection**: Offline tracking detection via Apple Find My advertisement classification (`AdClass::FindMy`).
- **Coexistence Instrumentation**: Radio duty-cycling and latency logging to ensure BLE telemetry link stability during Wi-Fi sniffing.

## 4. Passive Handshake & Credential Capture
- **Passive 4-Way Handshake Capture**: Reconstructing EAPOL 4-way handshakes without active packet injection (`HandshakeFrame`, `HandshakeComplete`).
- **Passive PMKID Capture**: Zero-copy extraction of PMKID from EAPOL Message 1 (`PmkidFound`).
- **Multi-Target PCAP & Hashcat 22000 Export**: Full standard PCAP file generation and Hashcat 22000 formatted export directly from web client.

## 5. Web Client Scanner & Survey Tools
- **Scanner Proximity & Station Filter**: Minimum RSSI filters (`> -80 dBm`, `> -70 dBm`, `> -60 dBm`) and active client filter (`WITH CLIENTS`).
- **Live Station Badges**: Visual count badges (`1 CLIENT`, `N CLIENTS`) on AP cards synced dynamically with `AssocStore`.
- **WiGLE Wardriving Export**: Integrated CSV export with browser Geolocation API coordinates for WiGLE.net uploads.
