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

## BLE monitor (device slice `ble_monitor`)
Off by default. When switched on, a passive NimBLE scan (25 % duty cycle) classifies each advertisement as **Fast Pair** (service data
0xFE2C), **iBeacon** (Apple type 0x02, length 0x15), **Proximity pairing** (Apple type 0x07, AirPods-style) or **Generic**, and the core
keeps per-class counts over the last minute (`BleStats` every 10 s). `BleSpike` fires when a class exceeds
`max(baseline x factor, baseline + 20)` after a 60 s warm-up (factor 2-20, default 3), at most once a minute per class. It shares the radio
with the phone link, so watch **BLE LOST** in Status after enabling it.

## Alerts feed and anomaly rules (device slice `anomaly`)
A chronological feed, newest first, coloured by severity (blue info, yellow warn, red alert), filterable by source. The device raises:

| Kind | Rule | Severity |
|---|---|---|
| New access point for a known network | a known SSID appears on a BSSID not seen with it before (not during the first 30 s) | warn |
| Same BSSID on two channels | heard on two channels within a minute; can be adjacent-channel leakage, so treat it as a hint | info |
| Signal jumped | RSSI moved by more than 20 dB between cycles | info |
| Network came back | missing for more than 5 cycles, then heard again | info |

Each kind is reported at most once per 30 s per BSSID. Deauth floods of 50/s or more and BLE spikes of 10x the baseline are red.

## Compare scans
A scan is saved when you stop scanning (or with **Snapshot now**) and holds only the networks heard *during that scan*. Pick two to see
new, disappeared, signal-changed (6 dB or more) and channel-changed networks. **Export JSON** saves the diff.

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
