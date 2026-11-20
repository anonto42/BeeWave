---
name: Feature Request
about: Suggest an idea or enhancement for BeeWave
title: "[FEATURE] "
labels: enhancement
assignees: ''
---

**Is your feature request related to a problem? Please describe.**
A clear and concise description of what the problem or use case is.

**Describe the Solution You'd Like**
A clear and concise description of the capability, telemetry message, or UI improvement you would like to see.

**Describe Alternatives You've Considered**
A description of any alternative solutions, features, or workarounds you've considered.

**Invariants & Scope Confirmation**
- [ ] This feature complies with BeeWave's strictly receive-only design (does not require packet transmission, deauth injection, or rogue AP broadcasting).
- [ ] Any core additions will maintain `#![no_std]` and zero dynamic heap allocation in `crates/core`.

**Additional Context**
Add any other context, 802.11 / BLE specifications, or screenshots about the feature request here.
