//! Hexagonal core: pure domain logic + port traits. No ESP-IDF, no BLE, no threads.
//!
//! ```text
//!  driving adapters (BLE cmd RX, sniffer frames, timer)         driven adapters (BLE notify, radio, flash, system)
//!            │                                                              ▲
//!            ▼                                                              │
//!        app::App  ── routes ──▶ features::{radio, scan, status, ota} ──▶ ports::*
//! ```
//! Each feature slice owns its state, rules and tests. Features depend on `ports`, never on each other.
//! Dependencies point inward only: adapters -> core -> protocol.
#![cfg_attr(not(feature = "std"), no_std)]

pub mod app;
pub mod features;
pub mod ports;
