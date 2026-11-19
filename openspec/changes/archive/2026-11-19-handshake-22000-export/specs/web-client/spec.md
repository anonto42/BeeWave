# Spec Delta

## ADDED Requirements

### Requirement: Four-way handshake Hashcat Mode 22000 export
The web client SHALL support synthesizing standard Hashcat Mode 22000 `WPA*02*` format hashes from captured EAPOL 4-way handshakes and exporting them as `.22000` text files.

#### Scenario: Complete handshake Hashcat export
- **WHEN** a 4-way handshake has both Message 1 (or Message 3) and Message 2 frames captured
- **THEN** the client can export a valid Hashcat mode 22000 hash line containing MIC, AP MAC, Station MAC, ESSID hex, ANonce, zeroed-MIC EAPOL frame, and message pair flag

#### Scenario: Incomplete handshake without ANonce or M2
- **WHEN** a handshake lacks Message 2 or lacks an ANonce source (Message 1 or 3)
- **THEN** mode 22000 export is unavailable or disabled for that handshake pair

### Requirement: One-click clipboard copy for Hashcat hashes
The web client SHALL provide a 1-click copy action for both PMKID (`WPA*01*`) and 4-way handshake (`WPA*02*`) Hashcat Mode 22000 hashes, copying the formatted hash line to the clipboard with momentary visual confirmation.

#### Scenario: Copying PMKID hash
- **WHEN** the user triggers the copy action on a captured PMKID record
- **THEN** the `WPA*01*...` line is written to the system clipboard and the button shows visual confirmation

#### Scenario: Copying handshake hash
- **WHEN** the user triggers the copy action on a captured 4-way handshake
- **THEN** the `WPA*02*...` line is written to the system clipboard and the button shows visual confirmation
