## ADDED Requirements

### Requirement: Latency statistics
The core SHALL aggregate latency samples into a fixed-size histogram without allocating and SHALL report upper bounds for the 50th and 99th percentiles and the maximum, returning no percentile when there are no samples.

#### Scenario: Slow tail
- **WHEN** two percent of samples are far slower than the rest
- **THEN** the 99th percentile reports the slow bucket

#### Scenario: Above the last bucket
- **WHEN** a sample exceeds the last bucket edge
- **THEN** the percentile reports the true maximum

### Requirement: Configurable hop order
The radio feature SHALL select the next hop channel from a supplied order, wrapping at the end, and SHALL restart from the first entry when the current channel is not in the order.

#### Scenario: Wrap
- **WHEN** the current channel is the last in the order
- **THEN** the next channel is the first in the order
