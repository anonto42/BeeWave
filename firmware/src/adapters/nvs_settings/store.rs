use beewave_core::ports::{Settings, SettingsError, SettingsStore, AUTO_CHANNEL, MAX_CHANNEL};
use beewave_protocol::{
    decode_telemetry, encode_telemetry, TelemetryMessage, HEADER_LEN, MAX_BODY, SETTINGS_SCHEMA,
};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, NvsDefault};
use esp_idf_svc::sys::EspError;

/// NVS namespace and keys (keys are limited to 15 characters).
const NAMESPACE: &str = "beewave";
/// All settings and the revision counter in ONE blob: an NVS blob write is atomic, so a power cut
/// leaves either the old settings or the new ones, never a mix. The blob is a wire `SettingsAll`
/// packet, which carries its own version byte and schema number.
const KEY_BLOB: &str = "settings";
/// The 32-byte device key a client provisioned (see the core's auth gate).
const KEY_AUTH: &str = "authkey";
/// Keys of the first release, read once to migrate (the pinned channel and scan-at-boot).
const KEY_CHANNEL: &str = "channel";
const KEY_AUTO_SCAN: &str = "autoscan";

const BUF: usize = HEADER_LEN + MAX_BODY + 8;

pub struct NvsSettings {
    nvs: EspNvs<NvsDefault>,
    /// How many saves succeeded since the last erase; persisted inside the blob.
    revision: u32,
}

impl NvsSettings {
    pub fn new(partition: EspDefaultNvsPartition) -> Result<Self, EspError> {
        Ok(Self {
            nvs: EspNvs::new(partition, NAMESPACE, true)?,
            revision: 0,
        })
    }

    /// The first-release values, if present, as a starting point.
    fn migrate_legacy(&mut self) -> Settings {
        let mut s = Settings::default();
        if let Some(c) = self.nvs.get_u8(KEY_CHANNEL).ok().flatten() {
            if c == AUTO_CHANNEL || (1..=MAX_CHANNEL).contains(&c) {
                s.radio.default_channel = c;
            }
        }
        if let Some(a) = self.nvs.get_u8(KEY_AUTO_SCAN).ok().flatten() {
            s.scan.scan_at_boot = a != 0;
        }
        s
    }
}

impl SettingsStore for NvsSettings {
    fn load(&mut self) -> Settings {
        let mut buf = [0u8; BUF];
        if let Ok(Some(raw)) = self.nvs.get_raw(KEY_BLOB, &mut buf) {
            if let Ok((
                _,
                TelemetryMessage::SettingsAll {
                    schema: SETTINGS_SCHEMA,
                    revision,
                    scan,
                    radio,
                    telemetry,
                    alerts,
                },
            )) = decode_telemetry(raw)
            {
                self.revision = revision;
                return Settings {
                    scan,
                    radio,
                    telemetry,
                    alerts,
                };
            }
        }
        self.revision = 0;
        self.migrate_legacy()
    }

    fn save(&mut self, s: &Settings) -> Result<(), SettingsError> {
        let revision = self.revision.wrapping_add(1);
        let msg = TelemetryMessage::SettingsAll {
            schema: SETTINGS_SCHEMA,
            revision,
            scan: s.scan.clone(),
            radio: s.radio.clone(),
            telemetry: s.telemetry.clone(),
            alerts: s.alerts.clone(),
        };
        let mut buf = [0u8; BUF];
        let n = encode_telemetry(0, &msg, &mut buf).map_err(|_| SettingsError::Storage)?;
        self.nvs
            .set_raw(KEY_BLOB, &buf[..n])
            .map_err(|_| SettingsError::Storage)?;
        self.revision = revision; // only after flash accepted it
        Ok(())
    }

    fn save_revision(&mut self) -> u32 {
        self.revision
    }

    fn load_key(&mut self) -> Option<[u8; 32]> {
        let mut buf = [0u8; 32];
        let got = self.nvs.get_raw(KEY_AUTH, &mut buf).ok().flatten()?;
        // anything but exactly 32 bytes is treated as "no key", never as a partial key
        (got.len() == 32).then_some(buf)
    }

    fn save_key(&mut self, key: &[u8; 32]) -> Result<(), SettingsError> {
        self.nvs
            .set_raw(KEY_AUTH, key)
            .map(|_| ())
            .map_err(|_| SettingsError::Storage)
    }

    fn erase_all(&mut self) -> Result<(), SettingsError> {
        for key in [KEY_BLOB, KEY_AUTH, KEY_CHANNEL, KEY_AUTO_SCAN] {
            self.nvs.remove(key).map_err(|_| SettingsError::Storage)?;
        }
        self.revision = 0;
        Ok(())
    }
}
