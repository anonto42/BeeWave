# Design: WiGLE.net Wardriving CSV Export & Geolocation

## Context
WiGLE (Wireless Geographic Logging Engine) requires a specific CSV format for wireless upload logs:
- **Header Line 1**: File version and client system metadata:
  `WigleWifi-1.4,appRelease=2.0.0,model=BeeWave-ESP32,release=2.0.0,device=BeeWave,display=BeeWave,board=ESP32,brand=BeeWave`
- **Header Line 2**: Field column names:
  `MAC,SSID,AuthMode,FirstSeen,Channel,Frequency,RSSI,CurrentLatitude,CurrentLongitude,AltitudeMeters,AccuracyMeters,Type`
- **Data Rows**:
  `MAC`: BSSID (colon-separated, lowercase or uppercase)
  `SSID`: Network SSID (quotes escaped for commas and quotes)
  `AuthMode`: Wi-Fi capability string, e.g. `[WPA2-PSK-CCMP][RSN][ESS]`
  `FirstSeen`: Formatted UTC timestamp `YYYY-MM-DD HH:MM:SS`
  `Channel`: Integer channel 1..=14
  `Frequency`: 2412 + 5 * (channel - 1) MHz (2484 MHz for ch 14)
  `RSSI`: Signal strength in dBm
  `CurrentLatitude`: Decimal degrees
  `CurrentLongitude`: Decimal degrees
  `AltitudeMeters`: Elevation in meters
  `AccuracyMeters`: Location accuracy in meters
  `Type`: Fixed string `"WIFI"`

## Architecture & Implementation

### 1. `GpsTracker` (`web/src/shared/gps/`)
- Pure class with zero dependencies outside browser APIs:
  - `start()` / `stop()`: Uses `navigator.geolocation.watchPosition` with `{ enableHighAccuracy: true, timeout: 10000, maximumAge: 1000 }`.
  - Exposes `coords: GpsCoordinates | null` (`lat`, `lon`, `alt`, `accuracy`, `at`).
  - `onChange(cb: () => void)` subscriber list for reactive UI updates.
  - Supports mock GPS injection in unit tests.

### 2. WiGLE Formatter (`web/src/shared/monitor/wigle.ts`)
- Pure function `networkToWigleAuthMode(sec?: SecurityRecord): string`:
  - Maps `WifiSecurityProtocol` (`open`, `wep`, `wpa`, `wpa2`, `wpa3`, `wpa2Wpa3`, `owe`, `enterprise`) to standard WiGLE capability tags.
  - Appends `[PMF-R]` or `[PMF-C]` when Protected Management Frames are detected.
  - Always appends `[ESS]` for infrastructure APs.
- Pure function `channelToFrequencyMhz(channel: number): number`:
  - 1..13 -> `2407 + channel * 5`
  - 14 -> `2484`
- Pure function `formatWigleTimestamp(epochMs: number): string`:
  - Generates `YYYY-MM-DD HH:MM:SS`.
- Pure function `generateWigleCsv(networks: readonly NetworkRecord[]): string`:
  - Builds metadata header, column header, and escaped data rows.

### 3. User Interface Integration
- **Scanner View**:
  - Adds `[GPS]` toggle status button in toolbar (e.g. `GPS OFF` / `GPS ON · ±5m`).
  - Adds `[WIGLE CSV]` download button in toolbar.
- **Settings View**:
  - Adds `WiGLE CSV (Wardriving)` option in Session export dropdown.
