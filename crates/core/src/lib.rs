//! Hexagonal core: pure domain logic + port traits. No ESP-IDF, no BLE, no threads.
//!
//! ```text
//!  driving adapters (BLE commands, sniffer frames, timer)      driven adapters (BLE notify, radio, flash, system)
//!            │                                                              ▲
//!            ▼                                                              │
//!        app::App  ── routes ──▶ features::{radio, scan, status, ota} ──▶ ports::*
//! ```
//! Dependencies point inward only: adapters -> core -> protocol.
//!
//! Where to look:
//! - `ports/`: the traits (what the core needs from the world), one file each
//! - `features/`: one folder per capability (types, logic, tests)
//! - `app/`: the service that composes the features
#![cfg_attr(not(feature = "std"), no_std)]

pub mod app;
pub mod features;
pub mod ports;
