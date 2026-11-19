# Proposal

## Why
Wireless penetration testers and wardriving enthusiasts map access point signal coverage, channel usage, and geographic security postures for network resilience research and site surveys. The standard global format for uploading wireless observations to geographic databases (such as WiGLE.net) is the WiGLE CSV v1.4/1.6 file format (`WigleWifi-1.4`), which correlates network MAC addresses, SSIDs, capabilities/auth modes, frequencies, and signal strengths with latitude, longitude, and elevation coordinates.

While BeeWave scans IEEE 802.11 networks, calculates RSSIs, and parses security capabilities (WPA2, WPA3, PMF, etc.), it currently only exports raw session JSON and generic tabular CSV. Users cannot export wardriving logs directly into WiGLE without external scripts or manual conversion.

## What Changes
- Implement `GpsTracker` in `web/src/shared/gps/` (using HTML5 `navigator.geolocation.watchPosition`) to track live geographic coordinates (latitude, longitude, altitude, accuracy) with user permission.
- Integrate GPS coordinates into `ScanStore` (`NetworkRecord` and `ScanEvent`), associating network observations with active coordinates.
- Implement `wigle.ts` in `web/src/shared/monitor/` providing WiGLE v1.4 compliant CSV generation (`WigleWifi-1.4`), formatting BSSIDs, SSIDs, WiGLE AuthMode capability strings (e.g. `[WPA2-PSK-CCMP][RSN][ESS]`), channels, frequencies (MHz), RSSI, timestamps, and coordinates.
- Add 1-click **WIGLE CSV** download buttons and GPS tracking controls in the Scanner view and the Settings Import/Export panel.
- Add unit tests for WiGLE capability string generation, channel-to-frequency mapping, CSV escaping, and GPS state tracking.

## Capabilities

### Modified Capabilities
- `web-client`: Acquire browser geolocation, stamp network sightings with GPS coordinates, and generate WiGLE v1.4 compliant CSV files for wardriving.

## Impact
- `crates/protocol`: Unchanged (wire protocol already provides BSSID, RSSI, channel, and security).
- `crates/core`: Unchanged.
- `web`: Adds `shared/gps/tracker.ts`, `shared/monitor/wigle.ts`, UI export buttons in Scanner and Settings, and unit tests in `web/test`.
