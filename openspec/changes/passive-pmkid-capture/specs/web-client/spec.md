# Spec Delta

## ADDED Requirements

### Requirement: PMKID display and Hashcat export
The client SHALL display captured PMKID values associated with targeted access points and SHALL offer export in standard Hashcat format (`WPA*01*...`).

#### Scenario: PMKID received for target
- **WHEN** a `PmkidFound` event matches a targeted BSSID
- **THEN** the PMKID is displayed in the handshake panel with a Hashcat export option
