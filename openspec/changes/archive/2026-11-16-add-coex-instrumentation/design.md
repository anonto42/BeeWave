Core stays clock-free: the firmware measures with `Instant`, the core only aggregates microsecond
samples. Percentiles are bucket upper edges (edges: 0.5, 1, 2, 5, 10, 20, 50, 100, 200, 500 ms), so a
"p99 <= 50 ms" check is exact at that edge. The overflow bucket reports the true maximum.
The hop order is a plain array in the Wi-Fi adapter; `next_in_order` is the pure step function.
Nothing here changes default behaviour: sequential hop order and the phone-chosen BLE parameters
are what shipped before.
