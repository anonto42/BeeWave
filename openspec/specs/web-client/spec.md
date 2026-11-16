# web-client Specification

## Purpose

An offline-capable PWA that connects to the device over Web Bluetooth and presents status, control and scan results through independent feature slices.

## Requirements

### Requirement: Transport abstraction
Features SHALL talk to the device only through the `Transport` port so the BLE implementation and the mock are interchangeable.

#### Scenario: Mock mode
- **WHEN** the page is opened with `?mock`
- **THEN** the UI runs against a simulated device without Bluetooth

### Requirement: Isolated feature slices
Each feature under `web/src/features/` SHALL depend only on `shared/` and MUST NOT import another feature.

#### Scenario: Adding a feature
- **WHEN** a new feature module is added and mounted in `main.ts`
- **THEN** no existing feature changes

### Requirement: Offline shell
The production build SHALL be cached by a service worker so the app starts without network access.

#### Scenario: Cold offline start
- **WHEN** the installed app is opened with no connectivity
- **THEN** the shell loads from the cache
