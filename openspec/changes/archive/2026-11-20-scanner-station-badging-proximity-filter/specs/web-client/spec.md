# Spec Delta: web-client

## ADDED Requirements

### Requirement: Scanner Proximity and Station Filtering
The web client SHALL provide a filter selector in the Scanner view allowing filtering of visible networks by minimum RSSI thresholds (`-80 dBm`, `-70 dBm`, `-60 dBm`) and by presence of connected client stations (`WITH CLIENTS`). The network count indicator SHALL display the filtered count alongside the total count when filtering is active.

#### Scenario: Proximity filter applied
- **WHEN** the user selects an RSSI threshold (e.g. `> -70 dBm`)
- **THEN** only networks with RSSI at or above that threshold are rendered in the network list

#### Scenario: Station filter applied
- **WHEN** the user selects `WITH CLIENTS` filter
- **THEN** only networks with one or more associated client stations in `AssocStore` are displayed

#### Scenario: Network count indicator with active filter
- **WHEN** filtering reduces the displayed networks from 10 to 3
- **THEN** the count indicator displays `3 / 10 networks`

### Requirement: Connected Station Badging in Scanner
The web client SHALL query `AssocStore` for each displayed Access Point and render a visual station count badge on the network card, reactively updating when new client associations are intercepted without requiring a manual page refresh.

#### Scenario: Access point with associated clients
- **WHEN** an Access Point has associated client stations tracked in `AssocStore`
- **THEN** the network card in the Scanner view renders a badge displaying the active client count (e.g. `1 CLIENT` or `N CLIENTS`)

#### Scenario: Reactive update on association telemetry
- **WHEN** an `assocResult` telemetry message is received for an Access Point currently visible in Scanner
- **THEN** the Access Point's station badge reactively updates to reflect the new client count
