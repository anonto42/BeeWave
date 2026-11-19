# Spec Delta

## ADDED Requirements

### Requirement: Unified target workspace and channel locking
The web client SHALL provide a 1-click action to pin the radio to a target's operating channel for reliable handshake capture, and SHALL present guidance when multiple targets operate across different channels.

#### Scenario: Pinning radio to target channel
- **WHEN** the user activates channel lock on a target operating on channel N
- **THEN** a configureRadio command is sent to the device to pin to channel N and the target indicates channel focus

#### Scenario: Multiple targets on distinct channels
- **WHEN** targets have distinct operating channels
- **THEN** the interface informs the user that the sniffer is hopping and provides per-target channel focus options

### Requirement: Crackable handshake state and early export
The web client SHALL evaluate whether captured handshake frames provide sufficient cryptographic material (M1+M2 or PMKID) for offline cracking, display clear status indicators, and enable export and copy actions immediately without requiring all 4 messages.

#### Scenario: M1 and M2 captured
- **WHEN** a handshake pair contains Message 1 (ANonce) and Message 2 (SNonce + MIC)
- **THEN** the status indicates crackable readiness and Hashcat mode 22000 export and copy actions are enabled

#### Scenario: Incomplete frames
- **WHEN** a handshake has only Message 1 without PMKID and without Message 2
- **THEN** the status indicates waiting for client response and export actions remain disabled
