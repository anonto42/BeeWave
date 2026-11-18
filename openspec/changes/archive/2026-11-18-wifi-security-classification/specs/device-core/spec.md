# Spec Delta: device-core

## ADDED Requirements

### Requirement: Zero-allocation Wi-Fi security suite and PMF parsing
The core scan engine SHALL parse 802.11 Capability Information flags, RSN elements (Tag 48), and WPA elements (Tag 221) from Beacon and Probe Response frames without heap allocation to extract security protocol, AKM, cipher suite, and Protected Management Frame (PMF) mode.

#### Scenario: Open network without privacy bit
- **WHEN** a beacon is received with Privacy bit cleared (0) and no RSN/WPA elements
- **THEN** the core identifies the security protocol as Open and PMF mode as None

#### Scenario: WPA2-PSK network with CCMP
- **WHEN** a beacon is received with an RSN element specifying AKM suite 00-0F-AC:2 (PSK) and pairwise cipher CCMP
- **THEN** the core identifies the security protocol as WPA2-Personal with CCMP cipher

#### Scenario: WPA3-SAE network with mandatory PMF
- **WHEN** a beacon is received with an RSN element specifying AKM suite 00-0F-AC:8 (SAE) and RSN capabilities with MFPR=1 and MFPC=1
- **THEN** the core identifies the security protocol as WPA3-Personal and PMF mode as Required

#### Scenario: WPA2/WPA3 transition mode
- **WHEN** a beacon is received with an RSN element specifying both PSK (00-0F-AC:2) and SAE (00-0F-AC:8) AKMs with MFPC=1 and MFPR=0
- **THEN** the core identifies the security protocol as WPA2/WPA3 Transition and PMF mode as Capable

### Requirement: Security observation deduplication
The core SHALL track observed security postures in a bounded deduplication table (`SecurityTracker`) and emit `SecurityResult` messages only when an Access Point is first observed or its security attributes (protocol, PMF, AKM, cipher) change.

#### Scenario: Duplicate beacons with identical security
- **WHEN** repeated beacons from the same BSSID are received with unchanged security and PMF attributes
- **THEN** the core does not re-emit `SecurityResult` messages over the BLE sink

#### Scenario: Security configuration change or downgrade
- **WHEN** a beacon for a known BSSID is received with altered security protocol or PMF attributes
- **THEN** the core immediately emits an updated `SecurityResult` message
