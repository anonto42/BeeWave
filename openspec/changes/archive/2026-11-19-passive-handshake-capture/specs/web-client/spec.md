## ADDED Requirements

### Requirement: Handshake capture panel
The client SHALL show captured BSSID/client pairs with which messages were seen, SHALL let the owner turn the capture on or off, and SHALL offer a hex export of the captured EAPOL bytes with any truncated message marked.

#### Scenario: Off by default
- **WHEN** the device has not been told to enable handshake capture
- **THEN** no handshake telemetry is requested or shown
