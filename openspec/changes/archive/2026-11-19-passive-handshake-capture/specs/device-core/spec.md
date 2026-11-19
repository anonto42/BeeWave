## ADDED Requirements

### Requirement: Receive-only handshake classification
The core SHALL classify a parsed EAPOL-Key frame as message 1, 2, 3 or 4 of the four-way handshake from its key-info bits, MUST NOT transmit anything, and SHALL discard frames that are not EAPOL-Key frames of a recognized message.

#### Scenario: Each message number
- **WHEN** key-info bits match message 1, 2, 3 or 4
- **THEN** the frame is classified accordingly

#### Scenario: Unrecognized key-info
- **WHEN** the bits match none of the four messages (for example a group-key frame)
- **THEN** the frame is not reported

### Requirement: Per-pair capture tracking
The core SHALL track which handshake messages have been seen for each BSSID/client pair in a bounded table and SHALL report completion once a message carrying the ANonce and the message carrying the SNonce and MIC have both been seen.

#### Scenario: Completion
- **WHEN** message 2 and either message 1 or message 3 have both been seen for a pair
- **THEN** `HandshakeComplete` is reported once for that pair
