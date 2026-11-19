# Spec Delta

## ADDED Requirements

### Requirement: Batch PCAP and consolidated Hashcat 22000 multi-target export
The web client SHALL support combining all captured 4-way handshakes into a single chronologically-ordered libpcap (`.pcap`) file and consolidating all crackable handshakes and clientless PMKIDs into a single unified Hashcat Mode 22000 file with 1-click download and clipboard actions.

#### Scenario: Exporting all handshakes to a single PCAP
- **WHEN** multiple 4-way handshakes have been captured across different BSSIDs or clients and the user requests all PCAP export
- **THEN** the client generates and downloads a single `.pcap` file containing all captured EAPOL frames from all pairs ordered chronologically

#### Scenario: Exporting all handshakes and PMKIDs to a single 22000 file
- **WHEN** multiple crackable handshakes or PMKIDs have been captured and the user requests all 22000 export
- **THEN** the client generates and downloads a consolidated `.22000` text file containing standard `WPA*02*` and `WPA*01*` formatted lines for all valid captures

#### Scenario: Copying all 22000 hashes to clipboard
- **WHEN** crackable handshakes or PMKIDs exist and the user clicks copy all 22000
- **THEN** the consolidated 22000 formatted lines are copied to the system clipboard with momentary visual confirmation feedback

#### Scenario: No crackable handshakes or PMKIDs
- **WHEN** no crackable handshakes or PMKIDs have been captured
- **THEN** batch export controls remain hidden or disabled in the workspace toolbar
