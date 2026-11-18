# Spec Delta

## ADDED Requirements

### Requirement: Passive PMKID extraction
The core SHALL inspect the Key Data field of parsed EAPOL-Key Message 1 frames for PMKID Key Data Encapsulation (KDE) or RSN Information Elements without transmitting any frames, and SHALL emit `PmkidFound` when a valid PMKID is detected.

#### Scenario: EAPOL-Key M1 with PMKID KDE
- **WHEN** an EAPOL-Key Message 1 contains a PMKID KDE (`00-0F-AC-04`)
- **THEN** the core extracts the 16-byte PMKID, BSSID, and client MAC address

#### Scenario: Frame without PMKID
- **WHEN** an EAPOL frame contains no PMKID KDE
- **THEN** no PMKID telemetry is emitted

### Requirement: PMKID deduplication
The core SHALL maintain a bounded cache of recently seen PMKIDs and SHALL suppress redundant telemetry emissions for previously observed PMKID entries.

#### Scenario: Duplicate PMKID
- **WHEN** an identical PMKID is seen again for the same BSSID
- **THEN** the duplicate frame is ignored and not republished
