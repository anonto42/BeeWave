# Monitoring panels

Everything here is **receive-only**. BeeWave never transmits deauthentication, probe or beacon frames, and the BLE scan is
passive (no scan requests). Nothing in this feature changes `REQUEST_CONN_PARAMS` or bonding.

Where the data comes from: the device reports scan results, status and alerts over BLE; the phone builds everything else from
that stream (`web/src/shared/monitor`). Panels in hidden tabs keep collecting.

| Tab | Panel | Data source |
|---|---|---|
| Scan | Control, Scanner, Targets | `Status`, `ScanResult` |
| Analyze | **Results** (expanded table) | `ScanResult`, local OUI lookup |
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
