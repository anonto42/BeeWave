<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/logo/crest-reversed.svg">
    <img src="assets/logo/crest-primary.svg" alt="BeeWave Crest Logo" width="160">
  </picture>
</p>

<h1 align="center">BeeWave</h1>

<p align="center">
  <strong>High-Performance 802.11 & BLE Security Monitor, Threat Detector & Wardriving PWA</strong><br>
  Strictly Receive-Only &bull; Hexagonal Rust Core &bull; ESP32-WROOM &bull; Web Bluetooth &bull; Zero App Store Friction
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2021_Edition-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/Target-ESP32--WROOM--32-red?logo=espressif" alt="ESP32 Target">
  <img src="https://img.shields.io/badge/Core-%23!%5Bno__std%5D-blue" alt="no_std Core">
  <img src="https://img.shields.io/badge/Frontend-Vite_PWA_+_TS-green?logo=vite" alt="Vite PWA">
  <img src="https://img.shields.io/badge/Transport-Web_Bluetooth-blueviolet?logo=bluetooth" alt="Web Bluetooth">
  <img src="https://img.shields.io/badge/Architecture-Hexagonal_Ports_&_Adapters-informational" alt="Hexagonal Architecture">
  <img src="https://img.shields.io/badge/RF_Mode-Strictly_Receive--Only-success" alt="Receive Only">
</p>

---

## Overview

**BeeWave** is an open-source wireless security auditing tool and RF monitor built for standard **ESP32-WROOM** hardware and operated in real time through an installable **Web Bluetooth Progressive Web App (PWA)**. 

BeeWave is designed with a strict **receive-only philosophy**: it passively sniffs 802.11 management/data frames and BLE advertisements without ever transmitting a single deauthentication, probe request, or beacon packet. It combines hardware-level wireless inspection, real-time threat detection, passive WPA2/WPA3 handshake capturing, and GPS-tagged wardriving into a zero-installation, host-testable architecture.

> [!IMPORTANT]
> **Strictly Receive-Only**: BeeWave does not inject packets, transmit deauth frames, or broadcast rogue APs. All telemetry is derived passively from standard promiscuous sniffer callbacks.

---

## Key Capabilities

### 1. Wi-Fi Reconnaissance & Physical Layer Inspection
- **PHY Generation Classification**: Detects 802.11b, 802.11g, Wi-Fi 4 (802.11n Tag 45 HT), and Wi-Fi 6 (802.11ax Tag 255 Ext 35 HE) generation standards with zero-copy beacon element parsing.
- **WPS Device Fingerprinting**: Extracts WPS version (1.0 vs 2.0), setup lockout status (`AP Setup Locked`), Push-Button Configuration (PBC) state, and hardware manufacturer/model details directly from beacon information elements without connecting.
- **Security & PMF Classification**: Identifies Open, WEP, WPA-Personal, WPA2-Personal/Enterprise, WPA3-SAE, and OWE protocols alongside Protected Management Frames (`PMF Required`, `Optional`, or `None`).
- **Hidden SSID Decloaking**: Automatically correlates directed client probe responses with uncloaked beacon BSSIDs to unveil hidden network names.
- **Client Probe Sniffing**: Intercepts client station probe requests, catalogs Preferred Network Lists (PNL), and maps devices to hardware vendors via local OUI lookup.
- **Client-to-AP Association Mapping**: Passively inspects 802.11 Data frame headers (`ToDS`/`FromDS`) to map active client stations to their connected Access Points via a bounded, zero-allocation LRU tracker (`AssocTracker`).

### 2. Real-Time Attack & Rogue AP Threat Detection
- **Deauthentication Flood Detection**: Sliding-window rate detector alerting when deauth frames exceed a configurable rate per second (`DeauthAlert`).
- **Disassociation Flood Detection**: Dedicated disassociation attack detection isolating forced client disconnections (`DisassocAlert`).
- **Beacon Flood Detection**: Alerts on rogue AP SSID spoofing and beacon flooding tools by evaluating frame burst rates and unique BSSID densities (`BeaconFloodAlert`).
- **Probe Request Flood Detection**: High-rate client probe request flood detection to unmask active network scanners (`ProbeFloodAlert`).
- **Beacon Timestamp (TSF) Rollback Detection**: Tracks 64-bit microsecond monotonic hardware clocks across beacons; negative time steps ($\ge 5\text{ s}$) instantly flag software fake APs (e.g. `airbase-ng`, `wifiphisher`) as `TsfRollback` anomalies.
- **KARMA / Rogue AP Mutation Detection**: Flags APs advertising multiple distinct SSIDs on the same physical BSSID (`KarmaSsidChange`).
- **Evil Twin Security Downgrade Detection**: Flags when an established network SSID is observed advertising a degraded security posture (e.g., WPA3 downgrading to Open or WEP).
- **Rapid Channel Hopping & Multi-BSSID Anomalies**: Identifies BSSIDs hopping channels within 60 seconds or unverified BSSIDs spoofing known corporate SSIDs.

### 3. Bluetooth Low Energy (BLE) Auditing & Tracker Detection
- **Advertisement Classification**: Real-time rate tracking and anomaly spike detection for Google Fast Pair, Apple iBeacon, Microsoft Proximity, and generic beacons.
- **Apple AirTag / Find My Tracker Detection**: Offline tracking detection via Apple Find My manufacturer advertisement classification (`0x004C`, type `0x12`), alerting on nearby stalking or tracker presence.
- **Coexistence Tuning**: Monitored radio time-slicing and latency logging (`coex_debug`) to maintain high-throughput BLE telemetry link stability during continuous 2.4 GHz promiscuous sniffing.

### 4. Passive Handshake & Credential Capture
- **Passive 4-Way Handshake Capture**: Reconstructs complete WPA/WPA2 4-way EAPOL handshakes heard over the air without packet injection.
- **Passive PMKID Capture**: Zero-copy extraction of PMKIDs from EAPOL Key Message 1 frames for offline cracking.
- **Multi-Target PCAP Export**: Generates standard `.pcap` files (libpcap 2.4 format with Ethernet/802.11 framing) per-target or consolidated across all monitored targets.
- **Consolidated Hashcat 22000 Export**: Exports all captured PMKIDs and 4-way handshakes into consolidated Hashcat mode 22000 hash files (`*.22000`) ready for offline testing.

### 5. Wardriving & Physical Site Survey Tools
- **Integrated GPS Wardriving**: Automatically tags scanned networks with browser Geolocation API coordinates and exports directly to WiGLE-compliant CSV files.
- **Proximity RSSI Filtering**: Filter visible networks by signal threshold (`> -80 dBm`, `> -70 dBm`, `> -60 dBm NEAR`) to cut out background RF noise during physical site walks.
- **Connected Client Filtering**: Instant filtering (`WITH CLIENTS`) to display only access points with active station traffic.
- **Real-Time 2.4 GHz Channel Analyzer**: Interactive SVG channel occupancy and activity heatmaps across non-overlapping channels (1, 6, 11) and the entire 2.4 GHz spectrum.

---

## Architectural Principles

```
                  +----------------------------------------------+
                  |         Web Bluetooth PWA (web/)             |
                  |  Vite + TypeScript + Feature Slices + SW     |
                  +-----------------------+----------------------+
                                          | Postcard wire codec
                                          v (BLE Characteristic)
                  +----------------------------------------------+
                  |           firmware/ (Composition Root)       |
                  |      ESP-IDF Adapters (Wi-Fi, BLE, NVS)      |
                  +-----------------------+----------------------+
                                          | Driving events & ports
                                          v
                  +----------------------------------------------+
                  |           crates/core (no_std)               |
                  | Domain Logic + State Machines + Anomaly Core |
                  +-----------------------+----------------------+
                                          | Shared wire types
                                          v
                  +----------------------------------------------+
                  |         crates/protocol (no_std)             |
                  | Source of Truth for Postcard Wire Protocol   |
                  +----------------------------------------------+
```

1. **Hexagonal Dependency Rule**: `adapters -> core -> protocol`. `crates/core` strictly adheres to `#![no_std]`, never imports `esp-idf`, uses `heapless` collections, and performs zero dynamic heap allocations.
2. **Host-Testable**: All core logic, anomaly detectors, and protocol codecs are thoroughly unit-tested on host machines (`make test`) without requiring ESP32 hardware.
3. **Lossy Telemetry by Contract**: The telemetry sink never blocks. If the BLE buffer saturates, dropped packets are counted and reported in `Status.dropped` rather than stalling the sniffer radio.
4. **Application-Layer Security**:
   - **Command Authentication**: Commands require an authenticated session via HMAC-SHA256 challenge-response using a 32-byte device key.
   - **Auto-Claim Protocol**: An unprovisioned device announces itself as unclaimed; the first connecting browser claims it and writes the key to NVS.
   - **Ed25519-Signed OTA**: Firmware updates require an Ed25519 signature verified over the image SHA-256 before slot switching.

---

## Hardware Requirements

| Component | Specification | Notes |
|---|---|---|
| **Microcontroller** | ESP32-WROOM-32 (Classic Xtensa Dual-Core) | 4 MB Flash, 520 KB SRAM, rev 1 or newer |
| **USB Interface** | CP2102, CH340, or CH9102 USB-to-UART | `/dev/ttyACM0` or `/dev/ttyUSB0` |
| **Antenna** | PCB trace or external IPEX antenna | Standard 2.4 GHz Wi-Fi + BLE 4.2/5.0 |
| **Client Device** | Android, macOS, Linux, Windows, or iOS | Android/Desktop: Chrome or Edge.<br>iOS: **Bluefy** browser (Safari lacks Web Bluetooth). |

---

## Quickstart Guide

### 1. Development Environment Setup
The repository includes a self-contained Nix flake and direnv setup.

```bash
# Clone the repository
git clone https://github.com/ananto30/beewave.git
cd beewave

# Enter the hermetic development shell (loads Rust, Espup, Node, pnpm)
nix develop
# or: direnv allow
```

### 2. Verify Host Tooling & Tests
You can run all host unit tests and launch the mock web client without physical hardware:

```bash
# Run host test suite (Rust unit tests + Vitest)
make test

# Full check (formatting, Rust tests, no_std check, web build, OpenSpec validation)
make check

# Launch the Web PWA with a simulated device in your browser
make web-mock
```

### 3. Flash Physical ESP32 Hardware
Connect your ESP32 board via USB:

```bash
# Verify attached hardware and serial port
make doctor

# One-time toolchain installation
make setup

# Build release firmware, flash to board, and open serial monitor
make fw-run
```

### 4. Connect to Web Bluetooth PWA
1. Open the hosted PWA or run your local dev server: `make web-dev`.
2. Tap **CONNECT** in the top navigation bar.
3. Select `BeeWave` from the browser Bluetooth chooser.
4. On first connection to a fresh device, the PWA automatically provisions an authentication key and claims the device.
5. Tap **START SCAN** to initiate real-time passive monitoring!

---

## Operational Workflows

### Wardriving with GPS Tagging
1. Open BeeWave on a mobile browser (Chrome on Android or Bluefy on iOS).
2. In the **Scan** tab, tap **GPS** to activate browser geolocation tracking.
3. Walk or drive through the survey area; the scanner records coordinates for every detected BSSID.
4. Tap **WIGLE CSV** in the toolbar to export a WiGLE-compatible CSV ready for direct upload to [wigle.net](https://wigle.net).

### Rogue AP & Evil Twin Hunting
1. Switch to the **Alerts** tab to observe the live threat feed.
2. In the **Scan** tab, apply the **FILTER** selector (`> -70 dBm` or `> -60 dBm`) to focus on proximate emitters.
3. If an attacker uses tools like `airbase-ng` or `wifiphisher`, BeeWave triggers an immediate high-severity alert for **TSF Rollback** or **KARMA SSID Mutation**.
4. If an evil twin attempts an authentication downgrade attack against a known network, an alert with severity `alert` is emitted.

### WPA2/WPA3 Handshake & Hashcat Cracking
1. Mark target networks by clicking the network card or tapping **PIN**.
2. In the **Scan > Handshake Capture** section, ensure the listener is active.
3. Filter by **WITH CLIENTS** in the Scanner toolbar to isolate APs with active client stations.
4. When stations authenticate, BeeWave captures PMKIDs and 4-way handshakes passively.
5. Tap **ALL .22000** or **ALL .PCAP** to export files directly into your downloads folder for analysis with Hashcat mode 22000:
   ```bash
   hashcat -m 22000 beewave-handshakes.22000 wordlist.txt
   ```

---

## Project Structure

```
├── .agents/                    # Agent knowledge base, memory & maintenance rules
├── assets/                     # Brand identity, SVG crests, and design assets
├── crates/
│   ├── protocol/               # Wire types & postcard codec (no_std)
│   └── core/                   # Domain features, ports, and App service (no_std)
│       └── src/features/       # radio, scan, anomaly, assoc, probe, deauth, ota, etc.
├── docs/                       # Technical runbooks, architecture, and monitoring guides
├── firmware/                   # ESP-IDF adapters, FreeRTOS runtime & main.rs
├── openspec/                   # Spec-driven change management & capability specs
├── scripts/                    # Asset generation & auxiliary build scripts
└── web/                        # Vite + TypeScript Web Bluetooth PWA
    ├── src/features/           # scanner, targets, alerts, settings, control, ota
    ├── src/shared/             # monitor store, protocol codec, BLE transport
    └── test/                   # Golden wire vectors & Vitest test suites
```

---

## Makefile Reference

| Target | Description |
|---|---|
| `make test` | Runs all host unit tests (`beewave-core`, `beewave-protocol`, and web Vitest) |
| `make check` | Full local CI gate: format check, host tests, `#![no_std]` build, web build, and specs |
| `make vectors` | Regenerates golden wire protocol test vectors shared with web tests |
| `make web-dev` | Starts local HTTPS dev server for Web Bluetooth development |
| `make web-mock` | Starts local dev server with simulated hardware (`/?mock`) |
| `make web-build` | Compiles production PWA bundle into `web/dist/` |
| `make fw-build` | Builds the release ESP32 firmware image |
| `make fw-flash` | Flashes the release firmware to `PORT=/dev/ttyACM0` |
| `make fw-run` | Builds, flashes, and attaches serial monitor in one command |
| `make fw-image` | Creates an Ed25519-signed OTA firmware binary (`dist/beewave-app.bin.sig`) |
| `make fw-unclaim` | Erases device key from NVS partition to reset claim status |

---

## Documentation Index

- [Architecture Reference](docs/ARCHITECTURE.md): Detailed ports & adapters hexagonal design and memory budgets.
- [Monitoring & Attack Guide](docs/MONITORING.md): Exhaustive guide to detection heuristics, alert thresholds, and telemetry formats.
- [End-to-End Usage](docs/USAGE.md): Complete walkthrough of setup, mobile pairing, and PWA operation.
- [Settings & NVS Guide](docs/SETTINGS.md): Schema, defaults, and persistence of runtime configurations.
- [Brand Guidelines](docs/BRANDING.md): Crest logo assets, color specifications, and vector regeneration.
- [Benchmarking Runbook](docs/BENCH_RUNBOOK.md): Radio coexistence and frame throughput performance metrics.

---

## Security & Ethical Use

> [!CAUTION]
> **Responsible Usage**: BeeWave is developed solely for authorized security auditing, defensive RF monitoring, educational research, and personal wireless reconnaissance. Never use BeeWave to monitor or capture traffic from networks or devices without explicit authorization from the owner.

BeeWave is strictly **non-transmitting** on the 802.11 physical layer: it cannot deauthenticate stations, disrupt RF communications, or spoof AP beacons.

---

## License

BeeWave is open-source software licensed under the [MIT License](LICENSE) (or project root license).
