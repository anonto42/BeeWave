# Monitoring panels

Everything here is **receive-only**. BeeWave never transmits deauthentication, probe or beacon frames, and the BLE scan is
passive (no scan requests). Nothing in this feature changes `REQUEST_CONN_PARAMS` or bonding.

Where the data comes from: the device reports scan results, status and alerts over BLE; the phone builds everything else from
that stream (`web/src/shared/monitor`). Panels in hidden tabs keep collecting.

| Tab | Panel | Data source |
|---|---|---|
| Scan | Control, Scanner, Targets, **Handshake capture** | `Status`, `ScanResult`, `HandshakeCaptureState`, `HandshakeFrame`, `HandshakeComplete` |
| Analyze | **Results** (expanded table) | `ScanResult`, local OUI lookup |
| Analyze | **Client stations (probes)** | `ProbeResult`, local OUI lookup |
| Analyze | **Channels** | `ScanResult` (client-side) |
| Analyze | **RSSI history** | `ScanResult` history per BSSID |
| Analyze | **Compare scans** | snapshots saved when a scan stops |
| Alerts | **Alerts** (feed) | `DeauthAlert`, `BleSpike`, `AnomalyAlert`, watchlist |
| Alerts | **Deauth detector** | `DeauthAlert`, `MonitorConfig` |
| Alerts | **BLE monitor** | `BleStats`, `BleSpike`, `MonitorConfig` |
| Settings | **Watchlist** (section) | `ScanResult` (rules run on the phone) |
| Settings | **Import/Export** (section) | everything the phone already received |

## Results table
SSID, BSSID, vendor, RSSI, channel, first seen, last seen and scan count. Tap a header to sort (tap again to flip); filter by SSID
substring and by channel. The vendor comes from a **small local OUI table** (best effort, no network lookup): unlisted prefixes show
"Unknown", and locally administered (random) addresses show "Randomized". Add rows in `web/src/shared/oui/table.ts`.

## Client stations and probe sniffing (device slice `probe`)
Client devices (smartphones, laptops) periodically broadcast 802.11 Probe Request management frames (`subtype 0x40`)
to discover nearby APs. The device sniffs these frames passively without sending any probe response, extracting the
client's transmitter MAC, requested SSID (or empty string for wildcard/broadcast probes), channel and RSSI.
A bounded table on the device deduplicates observations to conserve BLE bandwidth, re-reporting a client only when it
probes for a new SSID, moves by at least 6 dB, or after an elapsed interval.

In the **Analyze > Client stations** panel, the app displays:
- Client MAC and hardware vendor identification via the OUI table (or "Randomized" if the IEEE locally administered bit is set).
- Probed Networks list (the device's Preferred Network List, PNL).
- Active signal level (dBm / bars), probe count, and last seen timestamp.

## Client-to-AP association tracking (device slice `assoc`)
When clients are actively communicating with an Access Point, they exchange 802.11 Data and QoS Data frames
(`type 0x08`). By inspecting the `ToDS` and `FromDS` control bits in the MAC header:
- `ToDS=1, FromDS=0` (Client to AP): Transmitter Address (TA) is the client station, Receiver Address (RA) is the AP BSSID.
- `ToDS=0, FromDS=1` (AP to Client): Receiver Address (RA) is the client station, Transmitter Address (TA) is the AP BSSID.

The device core parses these headers zero-copy without allocating memory, rejects multicast and invalid addresses,
and tracks associations in a bounded 32-entry LRU table (`AssocTracker`). Reports (`AssocResult`) are deduplicated
and re-emitted only on significant signal movement (6 dB+) or after 5 seconds.

In the PWA:
- Under **Scan > Targets**: Each targeted Access Point lists its actively communicating stations with hardware vendor name, signal strength, and packet count.
- Under **Analyze > Client stations**: Probed stations indicate their active AP association (or "Unassociated").

## WPS device fingerprinting and lock detection (device slice `scan`)
Wi-Fi Protected Setup (WPS) information elements (Tag 221 / `0xDD` with vendor OUI `00:50:f2:04`) in 802.11 Beacon management frames broadcast key device hardware and security configuration attributes:
- **WPS Version & State**: Identifies WPS 1.0 vs WPS 2.0 (`0x20`) and whether WPS is configured.
- **Lockout Detection (`AP Setup Locked`)**: Indicates when an AP has locked out WPS PIN authentication (e.g., following failed brute-force attempts).
- **Push Button Configuration (`Selected Registrar`)**: Indicates whether WPS push-button pairing is currently open/active.
- **Hardware Metadata**: Extracts manufacturer name, device model name, and device model number directly from the beacon TLV fields without connecting or sending any frames.

The device core parses vendor elements zero-allocation in `#![no_std]`, tracks known WPS states in a 32-entry bounded table (`WpsTracker`), and only re-reports when attributes change, preserving BLE bandwidth.

In the PWA:
- Under **Scan > Scanner**: Each network displaying WPS features badges such as `WPS` / `WPS 2.0`, `· LOCKED`, and `· PBC`, and displays the detected device name or model.

## Wi-Fi security suite and PMF classification (device slice `scan`)
Beacon and Probe Response management frames advertise authentication suites, cipher suites, and 802.11w Protected Management Frames (PMF) configuration:
- **Capability Information (offset 34)**: The Privacy bit (bit 4) identifies whether frames are encrypted.
- **RSN Information Element (Tag 48 / `0x30`)**: Identifies modern Robust Security Networks (WPA2-PSK, WPA3-SAE, WPA2/WPA3 Transition mode, OWE Enhanced Open, and 802.1X Enterprise) and extracts PMF flags (`MFPC` Capable / `MFPR` Required).
- **WPA Information Element (Tag 221 / `0xDD` with OUI `00:50:f2:01`)**: Identifies legacy WPA1 networks.

The core parses these elements zero-allocation in `#![no_std]` and deduplicates observations in a bounded 32-entry table (`SecurityTracker`), only emitting `SecurityResult` over BLE when an AP is first observed or when its security posture changes (e.g., detecting an Evil Twin security downgrade).

In the PWA:
- Under **Scan > Scanner**: Each network displays color-coded security badges: `OPEN` (yellow warn), `WEP` (red alert), `WPA`, `WPA2`, `WPA3`, `WPA2/WPA3`, `OWE`, `802.1X`, and PMF indicators (`· PMF` when required, `· PMF OPT` when capable).

## Channels
Two bars per channel 1-13: access points, and **reports in the last 60 s**. The second series is a proxy for how busy a channel is,
not airtime utilisation: the device only sees beacons and only re-reports a network when it changes.

## RSSI history
A sparkline per network over scan cycles (a *cycle* is 4 status heartbeats, about one 13-channel hop lap). The **Moved 6 dB+** view
lists networks whose latest report moved by at least 6 dB. The device de-duplicates scan results (a known network is re-reported only
after a 6 dB move), so this view is that rule made visible: every extra history point *is* such a move.

## Deauth detector (device slice `deauth_detect`)
The sniffer callback counts deauthentication frames (subtype 0xC0) per BSSID in lock-free counters; once a second the core compares
them with the threshold (default 10 per second, 0 = off) and publishes `DeauthAlert { bssid, count }` for up to 4 BSSIDs. The panel
shows a live table with a sparkline of frames per second and lets you change the threshold (`SetMonitor`; kept in RAM, back to the
default after a reboot).

## Disassociation attack detection (device slice `deauth_detect`)
Wireless penetration testing tools (such as `aireplay-ng --deauth`, `mdk4 d`, or Marauder) frequently transmit 802.11 Disassociation management frames (subtype 0xA0) in addition to or instead of Deauthentication frames (0xC0) to kick client stations off networks or trigger reconnection handshakes.
The promiscuous sniffer callback counts disassociation frames per BSSID in a dedicated lock-free table without heap allocation. When the disassociation rate reaches or exceeds the configured threshold, the core publishes `DisassocAlert { bssid, count }`. In the PWA, disassociation alerts appear in the Alert feed (`[DISASSOC FLOOD] Disassociation flood from <BSSID> (<count>/s)`) and populate the active kicker list.

## Beacon flood detector (device slice `scan`)
Attack tools (such as Hydra-ESP's `attack_beacon_spam.c` or ESP32 Marauder) flood the 2.4 GHz band with dozens of fake beacons with spoofed BSSIDs to jam scanner lists. The device tracks beacon arrival velocity and unique BSSIDs in 1-second sliding windows without allocation.
When beacon rate reaches or exceeds the threshold (25+ frames/s) with at least 8 unique BSSIDs in a 1-second window, the device publishes `BeaconFloodAlert { rate, unique_bssids }`. Alerts are rate-limited to at most one every 5 seconds to prevent BLE saturation.
In the PWA, beacon flood alerts appear as critical red alerts in the feed and toast notifications with the observed rate and unique BSSID count.

## Probe request flood detector (device slice `probe`)
Penetration testing and scanner-jamming attack tools (such as `mdk4 p`, `mdk3 p`, or ESP32 Marauder probe spam) flood the 2.4 GHz spectrum with bursts of 802.11 Probe Request frames (`subtype 0x40`) with randomized MAC addresses and fake SSIDs to exhaust Access Point client association tables or blind wireless monitors.
The core tracks probe arrival velocity and unique client MACs in a 1-second sliding window without heap allocation.
When probe arrival rate reaches or exceeds the threshold (30+ frames/s) with at least 8 unique client MAC addresses in a 1-second window, the core emits `ProbeFloodAlert { rate, unique_clients }`. Alerts are rate-limited to at most one every 5 seconds to prevent BLE saturation.
In the PWA, probe request flood alerts appear as critical red alerts in the feed and toast notifications with the observed rate and unique client MAC count (`[PROBE FLOOD ATTACK] Probe request flood attack: <rate>/s (<unique> unique client MACs)`).

## Hidden SSID automatic decloaking (device slice `scan`)
Access points broadcasting "hidden" networks emit 802.11 Beacon frames with an empty or null-padded SSID tag (Tag 0). However, when an authorized client probes for the network or connects, the AP responds with directed Probe Response frames containing the uncloaked cleartext SSID.
- **Deduplication Bypass**: While normal beacons are deduplicated by RSSI movement (>= 6 dB) and scan interval, uncovering a hidden SSID is high-priority security intelligence. `ScanTable` detects when an AP previously heard with an empty SSID is observed with a non-empty SSID, immediately bypassing deduplication and emitting `DecloakedResult { bssid, ssid, channel, rssi }`.
- **State Preservation**: Once an AP's SSID is uncloaked, subsequent empty beacon frames from that AP cannot revert the entry back to hidden.
- **PWA Visualization**: The scanner displays a prominent `DECLOAKED` badge alongside the revealed SSID, and an informational alert is published to the Alert feed (`[DECLOAKED] Hidden network <BSSID> uncloaked as "<SSID>" on CH <channel>`).

## BLE monitor (device slice `ble_monitor`)
Off by default. When switched on, a passive NimBLE scan (25 % duty cycle) classifies each advertisement as **Fast Pair** (service data
0xFE2C), **iBeacon** (Apple type 0x02, length 0x15), **Proximity pairing** (Apple type 0x07, AirPods-style), **Find My / AirTag** (Apple type 0x12, offline finding) or **Generic**, and the core
keeps per-class counts over the last minute (`BleStats` every 10 s). `BleSpike` fires when a class exceeds
`max(baseline x factor, baseline + 20)` after a 60 s warm-up (factor 2-20, default 3), at most once a minute per class. It shares the radio
with the phone link, so watch **BLE LOST** in Status after enabling it.

## BLE flood and spam attack detector (device slice `ble_monitor`)
Penetration testing and prank tools (such as Hydra-ESP's `attack_bt_spam.c`, Flipper Zero, or Marauder) broadcast high-velocity bursts of spoofed BLE advertisements to disrupt nearby phones and computers with pairing popups (Apple Action Modals, Google FastPair prompts, SwiftPair).
While `BleMonitor` tracks rolling averages over 60-second windows, `BleFloodDetector` tracks advertisement arrival rates in a 1-second sliding window without heap allocations.
- **Velocity Threshold**: When total BLE advertisement arrival rate reaches or exceeds 30 packets per second, the detector identifies the dominant advertisement class and emits `BleFloodAlert { rate, class }`.
- **Bandwidth Protection**: Alerts are rate-limited to at most one every 5 seconds (`BLE_FLOOD_COOLDOWN_TICKS`) to prevent saturation of the BLE control connection.
- **PWA Visualization**: The PWA Alert feed flags high-severity alerts (`[BLE FLOOD ATTACK] Rapid BLE advertisement flood: <rate>/s (<class> spam)`) to instantly alert the operator of active Bluetooth jamming or spamming activity.

## Alerts feed and anomaly rules (device slice `anomaly`)
A chronological feed, newest first, coloured by severity (blue info, yellow warn, red alert), filterable by source. The device raises:

| Kind | Rule | Severity |
|---|---|---|
| Evil Twin security downgrade detected | an SSID previously observed with secure encryption (WPA, WPA2, WPA3, OWE, Enterprise) appears open or downgraded to WEP | alert |
| New access point for a known network | a known SSID appears on a BSSID not seen with it before (not during the first 30 s) | warn |
| Same BSSID on two channels | heard on two channels within a minute; can be adjacent-channel leakage, so treat it as a hint | info |
| Signal jumped | RSSI moved by more than 20 dB between cycles | info |
| Network came back | missing for more than 5 cycles, then heard again | info |

Each kind is reported at most once per 30 s per BSSID. Deauth floods of 50/s or more, disassociation floods, and BLE spikes of 10x the baseline are red.

## Compare scans
A scan is saved when you stop scanning (or with **Snapshot now**) and holds only the networks heard *during that scan*. Pick two to see
new, disappeared, signal-changed (6 dB or more) and channel-changed networks. **Export JSON** saves the diff.

## Wardriving and WiGLE.net CSV export (PWA `scanner` & `settings`)
For wireless site surveying and mapping network locations with global databases (e.g. [wigle.net](https://wigle.net)), the PWA integrates browser-side Geolocation with scan reporting:
- **Live Geolocation Tracking**: Tapping **GPS** in the Scanner toolbar requests browser location permission (`enableHighAccuracy`) and continuously tracks coordinates (latitude, longitude, elevation, and accuracy in meters).
- **Network Coordinate Stamping**: Each observed network record and event is stamped with active GPS coordinates upon observation.
- **WiGLE CSV v1.4 File Format**: Tapping **WIGLE CSV** in the Scanner toolbar (or selecting `WiGLE CSV (Wardriving)` under Settings > Import/Export) downloads a standard `WigleWifi-1.4` formatted CSV file. The file maps BSSIDs, SSIDs, WiGLE AuthMode capability strings (e.g. `[WPA2-PSK-CCMP][RSN][ESS]`), channels, frequencies (MHz), signal strengths, and geographic coordinates ready for direct WiGLE upload.

## Watchlist (phone only)
Pin BSSIDs in Settings > Watchlist (from the networks heard so far, by typing one, or with the PIN button on a Scanner row), each with a label and its own rules. Alerts: it **appears**, it has been **missing** for more than 5 cycles, or
it **moves** by 6 dB or more. Pins are kept in this browser's `localStorage`; the device never sees them, so there is no device-side watchlist.

## Handshake capture (device slice `handshake`, Scan tab, below Targets)
Per **target**: tap a network in the Scanner to target it (the existing Targets mechanism), then a **CAPTURE
HANDSHAKE** button appears for it. Tapping that button is the toolbar for that target — for now it has one action.
The device itself has a single on/off switch (`SetHandshakeCapture`), not one per network, so the panel keeps the
switch on exactly while at least one target has capture requested, and shows each target only the pairs whose BSSID
matches it (several clients handshaking with the same AP each get their own row).

When switched on, the sniffer's promiscuous filter also admits data frames; EAPOL-Key frames from a WPA/WPA2 four-way
handshake are classified by message number (1-4) from their key-info bits. **Nothing is transmitted to cause a
handshake.** If no client re-authenticates on its own nearby, there is nothing to capture; that is inherent to a
receive-only tool.

A pair is marked complete once it has a message carrying the ANonce (1 or 3) and the message carrying the SNonce and
MIC (2) — the minimum an offline cracking attempt needs; its row then offers **.CAP** and **.PCAP** downloads (the
same classic pcap format; the two extensions just name it differently). The file is built on the phone from the
captured EAPOL bytes, wrapped in synthetic 802.11 Data frame headers (direction inferred from which message it is)
and the constant LLC/SNAP header the device itself stripped off — so it is openable in Wireshark and readable by
tools such as aircrack-ng or hcxpcapngtool. The filename carries the network's SSID and BSSID (for example
`Home_50c7bfaabb01.pcap`), with the client's MAC appended only when more than one client's handshake was captured
for that target. Each `HandshakeFrame` carries the raw EAPOL bytes, capped at 100 bytes to fit one BLE packet; a
message longer than that is still classified but flagged **truncated**, and the exported file notes it (`orig_len`
cannot be known past the cap) — this is a detection and logging feature, not a guaranteed capture pipeline.

**What this explicitly does not do:** force a client to reconnect. A technique sometimes paired with handshake capture
sends spoofed deauthentication frames to knock a connected client off the network so it reconnects immediately. That is
an active disruption of someone's connection (and is blocked outright on 802.11w / WPA3 networks, which reject
unauthenticated deauth). BeeWave has no transmit path for management frames at all — see invariant 4 — and this
feature was built specifically without one; see `openspec/changes/passive-handshake-capture/proposal.md`.

### PMKID capture and Hashcat export
When an AP supports PMK caching (IEEE 802.11i), it embeds a **PMKID** inside the Key Data field of EAPOL-Key Message 1
(`KDE 00-0F-AC-04`). The core slice inspects Message 1 frames passively, parses the TLV list without memory allocation,
and emits `PmkidFound` for any new (BSSID, PMKID) pair (deduplicated in a bounded table).

When a PMKID is heard for a targeted network, the panel displays the client MAC and the 16-byte PMKID with a **.22000**
download button. The exported file contains a single standard Hashcat mode 22000 hash line:
`WPA*01*<pmkid>*<bssid>*<client>*<essid_hex>***` which can be supplied directly to `hashcat -m 22000` for offline password
auditing without needing a complete 4-way handshake or external capture converters.

### Multi-target PCAP and consolidated Hashcat 22000 export
When multiple networks or client handshakes are captured during an audit:
- **ALL .PCAP**: Combines all captured EAPOL packets from all monitored target pairs into a single, chronologically sequenced libpcap file (`beewave_handshakes_<timestamp>.pcap`). This multi-target capture can be analyzed directly in Wireshark or batch-converted with `hcxpcapngtool`.
- **ALL .22000**: Merges all crackable 4-way handshakes (`WPA*02*`) and clientless PMKIDs (`WPA*01*`) into a single multi-line Hashcat Mode 22000 dictionary attack file (`beewave_hashes_<timestamp>.22000`).
- **COPY ALL 22000**: Copies the entire consolidated 22000 hash text to the system clipboard in one click with visual confirmation.

## Export session
In Settings > Import/Export, one button saves the session in the chosen format (JSON, CSV or both), assembled on the phone from what the app has already seen: scan reports, alerts, log lines, key/link
events and saved scans. The CSV is one flat file with a `kind` column. Cells that a spreadsheet could read as formulas (starting with
`=`, `+`, `-` or `@`) are prefixed with an apostrophe, because SSIDs are attacker-controlled text.

## Trying it without the board
Open the PWA with `?mock`. `MockWorld` simulates access points (including a second AP for a known SSID), deauth bursts, BLE counts and
every anomaly kind, and honours the same thresholds and validation as the device.

## Bench checklist (needs the board and a phone)
1. Scan tab: start a scan; Analyze shows networks, channels and RSSI history; stop it and start again; Compare lists two scans.
2. Alerts > BLE monitor: turn on, wait a minute with a phone nearby; per-class counts appear; **BLE LOST** stays near 0. Turn it off again.
3. Alerts > Deauth: with a test network you own and a deauth source you are authorised to use, confirm an alert above the threshold. (Not
   tested on air yet: only the host tests and the mock have exercised this path.)
4. Settings > Import/Export > Export session: the file(s) download and open.
5. Scan tab > target a network you own or are authorised to test, then CAPTURE HANDSHAKE below Targets; let an
   already-connected client reconnect on its own (toggle its Wi-Fi, or walk out of range and back). Two dots light up
   and the row offers .CAP/.PCAP once complete; open the file in Wireshark to confirm it holds the EAPOL frames. With
   no client reconnecting, nothing appears — that is expected, not a fault.
