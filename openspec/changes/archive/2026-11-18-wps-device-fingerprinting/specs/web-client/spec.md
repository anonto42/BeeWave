# Spec Delta

## ADDED Requirements

### Requirement: WPS and Hardware Details Display
The web client SHALL ingest `WpsResult` telemetry messages, storing WPS security posture and device identification per Access Point. The web client SHALL display WPS status badges (e.g., WPS version, Locked, Open, PushButton) and hardware manufacturer/model details in Access Point scanner rows and target details.

#### Scenario: Ingesting WPS metadata
- **WHEN** a `WpsResult` message is received for an Access Point BSSID
- **THEN** the Access Point record is updated with its WPS version, lockout state, and hardware details

#### Scenario: Displaying WPS audit badge
- **WHEN** an Access Point has WPS enabled
- **THEN** a WPS badge indicates whether WPS is locked against PIN attacks or active
