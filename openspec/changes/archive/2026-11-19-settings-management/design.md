## Validation

`validate(&Settings)` checks every range and these rules, and reports the first failing field as a `RejectReason`:
a custom hop list holds channels 1-13 once each (0 = unused) and at least one; `deauth_window_ms >= 2 x scan_interval_ms`.
Because the window tops out at 5000 ms, a scan interval above 2500 ms is always rejected (`WindowTooShort`).
`SaveSettingsAll` validates first, then persists, then adopts: a rejected or failed save leaves NVS and the live values untouched.

## Taking effect

Scan policy and alert threshold apply at once; the hop dwell and order are handed to the driver when a scan starts; the
status period and deauth window are read by the firmware loop each time it schedules; the default channel is applied on save.

## Factory reset

Accepted only when authenticated and no update is in flight (otherwise `Rejected { Busy }`); erases settings, the claimed key
and the revision, then reboots.

## Self-test

Refused while scanning (`busy`). Radio: tune to channel 1 then 13 and restore the pinned channel; NVS: write, read and erase
a scratch key; BLE: advertising or connected; OTA slot: ota_0 or ota_1 running and otadata coherent.
