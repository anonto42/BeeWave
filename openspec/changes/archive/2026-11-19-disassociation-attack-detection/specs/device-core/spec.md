# Spec Delta

## ADDED Requirements

### Requirement: Disassociation frame attack detection
The core SHALL track received 802.11 Disassociation management frames (subtype `0xA0`) per BSSID over a 1-second sliding window without heap allocation and SHALL raise `DisassocAlert` for each BSSID whose rate meets or exceeds the configured threshold.

#### Scenario: Disassociation threshold crossed
- **WHEN** disassociation frames for a BSSID reach or exceed the threshold within the 1-second window
- **THEN** a `DisassocAlert` carrying the BSSID and frame count is emitted over the telemetry sink

#### Scenario: Disassociation below threshold or disabled
- **WHEN** the disassociation count for a BSSID is below the threshold, or the threshold is 0
- **THEN** no `DisassocAlert` is emitted

#### Scenario: Multiple BSSIDs targeted
- **WHEN** multiple BSSIDs exceed the threshold within the same window
- **THEN** an alert is emitted for each qualifying BSSID up to the sink capacity
