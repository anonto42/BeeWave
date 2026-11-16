## ADDED Requirements

### Requirement: Signed update upload
The client SHALL send the image's signature file with the update, SHALL stop immediately with the device's reason when the device reports an OTA error, and MUST only report success after the device confirms the signed end.

#### Scenario: Refused signature
- **WHEN** the device reports a bad signature
- **THEN** the upload fails at once with that message instead of waiting for a timeout
