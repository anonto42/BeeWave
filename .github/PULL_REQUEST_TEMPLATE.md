## Description
Provide a concise explanation of the changes introduced in this pull request and why they are needed.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Refactoring or architectural improvement
- [ ] Documentation update
- [ ] Build / CI improvement

## Architecture & Safety Checklist
- [ ] **Receive-Only Guarantee**: This change does not transmit 802.11 frames or bypass receive-only constraints.
- [ ] **Core Invariants**: `crates/core` remains strictly `#![no_std]`, uses heapless collections, and performs zero heap allocation.
- [ ] **Dependency Rule**: Hexagonal boundary maintained (`adapters -> core -> protocol`).
- [ ] **Wire Protocol**: If wire messages changed, vectors were regenerated (`make vectors`) and tests pass.
- [ ] **Testing**: `make check` passes cleanly (fmt, host tests, no_std build, web build, and specs).
- [ ] **Git Hygiene**: Conventional commits used, no AI attribution lines included.
