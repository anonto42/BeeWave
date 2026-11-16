use core::ffi::c_void;
use core::ptr;

use beewave_core::ports::{OtaError, OtaWriter};
use esp_idf_svc::sys::{self, esp};
use sha2::{Digest, Sha256};

use super::session::Session;

#[derive(Default)]
pub struct FlashOta {
    session: Option<Session>,
}

// SAFETY: `Session.partition` points into the static, read-only partition table owned by ESP-IDF,
// so moving it to another thread is fine.
unsafe impl Send for FlashOta {}

impl OtaWriter for FlashOta {
    fn begin(&mut self, size: u32) -> Result<(), OtaError> {
        self.abort(); // restart cleanly if a previous update was in flight

        // SAFETY: null asks ESP-IDF for the next slot after the running one.
        let partition = unsafe { sys::esp_ota_get_next_update_partition(ptr::null()) };
        if partition.is_null() {
            return Err(OtaError::Flash);
        }
        // SAFETY: non-null pointer into the static partition table.
        if size as usize > unsafe { (*partition).size } as usize {
            return Err(OtaError::TooLarge);
        }

        let mut handle: sys::esp_ota_handle_t = 0;
        // SAFETY: `partition` is valid; `handle` is a valid out-pointer.
        esp!(unsafe { sys::esp_ota_begin(partition, size as usize, &mut handle) })
            .map_err(|_| OtaError::Flash)?;

        self.session = Some(Session {
            handle,
            partition,
            hasher: Sha256::new(),
        });
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> Result<(), OtaError> {
        let s = self.session.as_mut().ok_or(OtaError::BadState)?;
        // SAFETY: `data` is a valid slice for the duration of the call.
        esp!(unsafe { sys::esp_ota_write(s.handle, data.as_ptr() as *const c_void, data.len()) })
            .map_err(|_| OtaError::Flash)?;
        s.hasher.update(data);
        Ok(())
    }

    fn finish(&mut self, expected_sha256: &[u8; 32]) -> Result<(), OtaError> {
        let s = self.session.take().ok_or(OtaError::BadState)?;
        let digest: [u8; 32] = s.hasher.finalize().into();
        if &digest != expected_sha256 {
            // SAFETY: handle came from a successful esp_ota_begin.
            unsafe { sys::esp_ota_abort(s.handle) };
            return Err(OtaError::HashMismatch);
        }
        // esp_ota_end validates the image header and checksum, and frees the handle either way.
        esp!(unsafe { sys::esp_ota_end(s.handle) }).map_err(|_| OtaError::Flash)?;
        esp!(unsafe { sys::esp_ota_set_boot_partition(s.partition) }).map_err(|_| OtaError::Flash)
    }

    fn abort(&mut self) {
        if let Some(s) = self.session.take() {
            // SAFETY: handle came from a successful esp_ota_begin.
            unsafe { sys::esp_ota_abort(s.handle) };
        }
    }
}

impl Drop for FlashOta {
    fn drop(&mut self) {
        self.abort();
    }
}
