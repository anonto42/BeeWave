use beewave_protocol::*;

fn s<const N: usize>(x: &str) -> heapless::String<N> {
    x.try_into().unwrap()
}

fn sample_scan() -> ScanSettings {
    ScanSettings {
        scan_at_boot: true,
        scan_interval_ms: 500,
        dedup_rssi_db: 6,
        ssid_filter: s("Cafe"),
        channel_filter_mask: 0b1_0000_0100_0001,
    }
}

fn sample_radio() -> RadioSettings {
    RadioSettings {
        default_channel: 0,
        hop_dwell_ms: 250,
        hop_order: HopOrder::Custom,
        custom_hop_list: [1, 6, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    }
}

fn sample_telemetry() -> TelemetrySettings {
    TelemetrySettings {
        status_period_ms: 1000,
        log_level: LogLevel::Info,
    }
}

fn sample_alerts() -> AlertSettings {
    AlertSettings {
        deauth_threshold: 10,
        deauth_window_ms: 1000,
    }
}

#[test]
fn roundtrip_and_emit_golden_vectors() {
    let tele = [
        TelemetryMessage::Status {
            active_channel: 6,
            free_heap: 123_456,
            cpu_load: 42,
            dropped: 300,
            scanning: true,
        },
        TelemetryMessage::ScanResult {
            bssid: [1, 2, 3, 4, 5, 6],
            channel: 11,
            rssi: -67,
            ssid: s("Café-WiFi"),
        },
        TelemetryMessage::Event {
            code: 7,
            message: s("hello"),
        },
        TelemetryMessage::OtaAck { next_offset: 4096 },
        TelemetryMessage::Settings {
            channel: 6,
            auto_scan: true,
        },
        TelemetryMessage::AuthChallenge { nonce: [7; 16] },
        TelemetryMessage::AuthResult { ok: true },
        TelemetryMessage::Unclaimed,
        TelemetryMessage::DeauthAlert {
            bssid: [1, 2, 3, 4, 5, 6],
            count: 42,
        },
        TelemetryMessage::BleStats {
            fast_pair: 1,
            ibeacon: 2,
            proximity: 300,
            generic: 4000,
        },
        TelemetryMessage::BleSpike {
            class: AdClass::Proximity,
            count: 500,
            baseline: 40,
        },
        TelemetryMessage::AnomalyAlert {
            kind: AnomalyKind::RssiOutlier,
            bssid: [9, 8, 7, 6, 5, 4],
            detail: 27,
        },
        TelemetryMessage::MonitorConfig {
            deauth_threshold: 10,
            ble_enabled: true,
            ble_spike_factor: 3,
        },
        TelemetryMessage::SettingsAll {
            schema: SETTINGS_SCHEMA,
            revision: 300,
            scan: sample_scan(),
            radio: sample_radio(),
            telemetry: sample_telemetry(),
            alerts: sample_alerts(),
        },
        TelemetryMessage::SettingsSaved {
            revision: 7,
            ok: false,
            reason: Some(RejectReason::WindowTooShort),
        },
        TelemetryMessage::SettingsSaved {
            revision: 8,
            ok: true,
            reason: None,
        },
        TelemetryMessage::StorageInfo {
            nvs_used: 3000,
            nvs_free: 21576,
            ota0_active: true,
            ota1_active: false,
            ota0_size: 1_572_864,
            ota1_size: 1_572_864,
            settings_revision: 9,
        },
        TelemetryMessage::SelfTestResult {
            radio: true,
            nvs: true,
            ble: false,
            ota_slot: true,
            busy: false,
        },
        TelemetryMessage::DeviceInfo {
            version: s("0.1.0"),
            git_sha: s("a1b2c3d"),
            built: s("2026-11-18T10:00:00Z"),
            ota_key_fp: [0xAB; 16],
            request_conn_params: false,
            coex_debug: false,
            bonding: false,
        },
        TelemetryMessage::Rejected {
            reason: RejectReason::Busy,
        },
        TelemetryMessage::HandshakeCaptureState { enabled: true },
        TelemetryMessage::HandshakeFrame {
            bssid: [9; 6],
            client: [8; 6],
            message: 2,
            replay_counter: 42,
            truncated: false,
            eapol: heapless::Vec::from_slice(&[0xAA; 20]).unwrap(),
        },
        TelemetryMessage::HandshakeComplete {
            bssid: [9; 6],
            client: [8; 6],
        },
        TelemetryMessage::HandshakeFrame {
            bssid: [9; 6],
            client: [8; 6],
            message: 3,
            replay_counter: u64::MAX,
            truncated: true,
            eapol: heapless::Vec::from_slice(&[0xBB; MAX_EAPOL_CAPTURE]).unwrap(),
        },
        TelemetryMessage::PmkidFound {
            bssid: [9; 6],
            client: [8; 6],
            pmkid: [0x42; 16],
        },
    ];
    let cmds = [
        DeviceCommand::StartScan { band_mask: 3 },
        DeviceCommand::StopOperation,
        DeviceCommand::ConfigureRadio { channel: 13 },
        DeviceCommand::RebootDevice,
        DeviceCommand::OtaBegin {
            size: 1_000_000,
            sha256: [9; 32],
        },
        DeviceCommand::OtaChunk {
            offset: 96,
            data: heapless::Vec::from_slice(&[1, 2, 3]).unwrap(),
        },
        DeviceCommand::OtaEnd,
        DeviceCommand::GetSettings,
        DeviceCommand::SetSettings {
            channel: 0,
            auto_scan: true,
        },
        DeviceCommand::AuthBegin,
        DeviceCommand::AuthProve { mac: [5; 32] },
        DeviceCommand::OtaEndSigned {
            sig_r: [3; 32],
            sig_s: [4; 32],
        },
        DeviceCommand::Provision { key: [6; 32] },
        DeviceCommand::GetMonitor,
        DeviceCommand::SetMonitor {
            deauth_threshold: 10,
            ble_enabled: true,
            ble_spike_factor: 3,
        },
        DeviceCommand::GetSettingsAll,
        DeviceCommand::SaveSettingsAll {
            scan: sample_scan(),
            radio: sample_radio(),
            telemetry: sample_telemetry(),
            alerts: sample_alerts(),
        },
        DeviceCommand::FactoryReset,
        DeviceCommand::RunSelfTest,
        DeviceCommand::GetStorageInfo,
        DeviceCommand::GetDeviceInfo,
        DeviceCommand::SetHandshakeCapture { enabled: true },
        DeviceCommand::GetHandshakeCapture,
    ];
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let mut buf = [0u8; MAX_BODY + 2];
    let mut tv = vec![];
    for (i, t) in tele.iter().enumerate() {
        let n = encode_telemetry(i as u8, t, &mut buf).unwrap();
        assert_eq!(decode_telemetry(&buf[..n]).unwrap(), (i as u8, t.clone()));
        tv.push(serde_json::json!({ "hex": hex(&buf[..n]), "msg": format!("{t:?}") }));
    }
    let mut cv = vec![];
    for (i, c) in cmds.iter().enumerate() {
        let n = encode_command(i as u8, c, &mut buf).unwrap();
        assert_eq!(decode_command(&buf[..n]).unwrap(), (i as u8, c.clone()));
        cv.push(serde_json::json!({ "hex": hex(&buf[..n]), "msg": format!("{c:?}") }));
    }
    assert_eq!(decode_telemetry(&[9, 0, 0]), Err(Error::BadVersion(9)));
    let out = serde_json::json!({ "telemetry": tv, "commands": cv });
    std::fs::create_dir_all("../../web/test").unwrap();
    std::fs::write(
        "../../web/test/vectors.json",
        serde_json::to_string_pretty(&out).unwrap(),
    )
    .unwrap();
}

/// The firmware stores `SettingsAll` as one NVS blob, so the largest possible value must fit and round-trip.
#[test]
fn the_largest_settings_all_fits_the_storage_buffer_and_round_trips() {
    let msg = TelemetryMessage::SettingsAll {
        schema: SETTINGS_SCHEMA,
        revision: u32::MAX,
        scan: ScanSettings {
            scan_at_boot: true,
            scan_interval_ms: u32::MAX,
            dedup_rssi_db: u8::MAX,
            ssid_filter: s("0123456789abcdef0123456789abcdef"),
            channel_filter_mask: u16::MAX,
        },
        radio: RadioSettings {
            default_channel: u8::MAX,
            hop_dwell_ms: u16::MAX,
            hop_order: HopOrder::Custom,
            custom_hop_list: [255; HOP_LIST_LEN],
        },
        telemetry: TelemetrySettings {
            status_period_ms: u32::MAX,
            log_level: LogLevel::Debug,
        },
        alerts: AlertSettings {
            deauth_threshold: u8::MAX,
            deauth_window_ms: u16::MAX,
        },
    };
    // the buffer the firmware uses: header + MAX_BODY + 8
    let mut buf = [0u8; HEADER_LEN + MAX_BODY + 8];
    let n = encode_telemetry(0, &msg, &mut buf).unwrap();
    assert!(n < buf.len() - 8, "{n} bytes leaves no headroom");
    assert_eq!(decode_telemetry(&buf[..n]).unwrap(), (0, msg));
}
