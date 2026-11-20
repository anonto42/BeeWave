# web-client Specification Delta

## ADDED Requirements

### Requirement: PHY Standard Badging in Scanner
The web client SHALL ingest `PhyResult` telemetry messages, store the PHY generation standard on the corresponding `NetworkRecord`, and render visible badges indicating the physical standard in the Scanner network cards.

#### Scenario: Visual badging of Wi-Fi generations
- **WHEN** a network has a classified PHY standard
- **THEN** the Scanner card displays a badge indicating the standard (e.g. `AX`, `N`, `802.11g`, or `802.11b`)
