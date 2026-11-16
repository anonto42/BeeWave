## Why

Wi-Fi sniffing and BLE share one radio, so notification latency can spike while hopping. We need
measurements from the board, and tuning knobs to act on them, without guessing values first.

## What Changes

- A latency histogram in the core (fixed buckets, percentile queries) so statistics are host-tested.
- Telemetry is stamped when queued; the `coex_debug` firmware feature logs queue-wait and notify-call
  percentiles every 10 seconds. Off by default.
- Channel hopping follows a configurable order, with the order logic in the core.
- Dwell time, hop order, queue depths and BLE connection parameters are named constants. Connection
  parameters are only requested when `REQUEST_CONN_PARAMS` is turned on.

## Non-goals

Choosing the final tuning values (needs board measurements), changing task priorities.

## Impact

`crates/core` (latency, radio slice), firmware runtime/BLE/Wi-Fi adapters, Makefile `FEATURES`.
RAM: one timestamp per telemetry queue slot (about 0.5 KB). Flash: negligible.
