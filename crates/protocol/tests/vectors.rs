use beewave_protocol::*;

fn s<const N: usize>(x: &str) -> heapless::String<N> {
    x.try_into().unwrap()
}

#[test]
fn roundtrip_and_emit_golden_vectors() {
    let tele = [
        TelemetryMessage::Status {
            active_channel: 6,
            free_heap: 123_456,
            cpu_load: 42,
            dropped: 300,
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
