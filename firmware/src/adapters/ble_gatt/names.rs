//! Short names for log lines. Names only: never payloads (keys, OTA data).
use beewave_protocol::{DeviceCommand, TelemetryMessage};

pub fn command_name(c: &DeviceCommand) -> &'static str {
    match c {
        DeviceCommand::StartScan { .. } => "start_scan",
        DeviceCommand::StopOperation => "stop",
        DeviceCommand::ConfigureRadio { .. } => "configure_radio",
        DeviceCommand::RebootDevice => "reboot",
        DeviceCommand::OtaBegin { .. } => "ota_begin",
        DeviceCommand::OtaChunk { .. } => "ota_chunk",
        DeviceCommand::OtaEnd => "ota_end",
        DeviceCommand::GetSettings => "get_settings",
        DeviceCommand::SetSettings { .. } => "set_settings",
        DeviceCommand::AuthBegin => "auth_begin",
        DeviceCommand::AuthProve { .. } => "auth_prove",
        DeviceCommand::OtaEndSigned { .. } => "ota_end_signed",
        DeviceCommand::Provision { .. } => "provision",
        DeviceCommand::GetMonitor => "get_monitor",
        DeviceCommand::SetMonitor { .. } => "set_monitor",
        DeviceCommand::GetSettingsAll => "get_settings_all",
        DeviceCommand::SaveSettingsAll { .. } => "save_settings_all",
        DeviceCommand::FactoryReset => "factory_reset",
        DeviceCommand::RunSelfTest => "run_self_test",
        DeviceCommand::GetStorageInfo => "get_storage_info",
        DeviceCommand::GetDeviceInfo => "get_device_info",
        DeviceCommand::SetHandshakeCapture { .. } => "set_handshake_capture",
        DeviceCommand::GetHandshakeCapture => "get_handshake_capture",
    }
}

/// `Some(name)` for the rare messages worth a log line (not the 1 Hz status or scan results).
pub fn notable_telemetry(m: &TelemetryMessage) -> Option<&'static str> {
    match m {
        TelemetryMessage::AuthChallenge { .. } => Some("auth_challenge"),
        TelemetryMessage::AuthResult { ok: true } => Some("auth_result ok"),
        TelemetryMessage::AuthResult { ok: false } => Some("auth_result REFUSED"),
        TelemetryMessage::Unclaimed => Some("unclaimed"),
        TelemetryMessage::Event { .. } => Some("event"),
        TelemetryMessage::DeauthAlert { .. } => Some("deauth_alert"),
        TelemetryMessage::BleSpike { .. } => Some("ble_spike"),
        TelemetryMessage::AnomalyAlert { .. } => Some("anomaly_alert"),
        TelemetryMessage::MonitorConfig { .. } => Some("monitor_config"),
        TelemetryMessage::SettingsSaved { ok: true, .. } => Some("settings_saved ok"),
        TelemetryMessage::SettingsSaved { ok: false, .. } => Some("settings_saved REFUSED"),
        TelemetryMessage::SelfTestResult { .. } => Some("self_test_result"),
        TelemetryMessage::Rejected { .. } => Some("rejected"),
        TelemetryMessage::HandshakeCaptureState { enabled } => Some(if *enabled {
            "handshake_capture on"
        } else {
            "handshake_capture off"
        }),
        TelemetryMessage::HandshakeComplete { .. } => Some("handshake_complete"),
        _ => None,
    }
}
