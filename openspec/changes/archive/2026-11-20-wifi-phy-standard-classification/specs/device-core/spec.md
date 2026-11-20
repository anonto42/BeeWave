# device-core Specification Delta

## ADDED Requirements

### Requirement: 802.11 PHY Standard Extraction and Deduplication
The scan feature SHALL parse 802.11 beacon and probe response frames to detect HT Capabilities (Element 45), ERP Information (Element 42), HE Capabilities (Element 255 Ext 35), and Supported Rates (Elements 1 and 50) without allocating, classify the physical standard into `WifiPhyStandard`, and emit `PhyResult` deduplicated in a bounded table (`PhyTracker`).

#### Scenario: 802.11n Wi-Fi 4 detection
- **WHEN** a beacon contains Information Element 45 (HT Capabilities)
- **THEN** the detected PHY standard is classified as `WifiPhyStandard::N` and emitted via `PhyResult`

#### Scenario: Deduplicated PHY reporting
- **WHEN** an Access Point is observed again with an unchanged PHY standard
- **THEN** `PhyTracker` suppresses duplicate telemetry emission
