# Data Model: UDP Data Forwarding (034-udp-forward)

**Date**: 2026-02-10
**Branch**: `034-udp-forward`

## Entities

### ForwardSettings (existing — no changes)

Already defined in `src/settings.rs`. Used as-is.

| Field      | Type          | Description                                    |
| ---------- | ------------- | ---------------------------------------------- |
| enabled    | bool          | Master toggle for forwarding                   |
| address    | Vec\<String\> | Destination IP addresses                       |
| port       | Vec\<u16\>    | Destination ports (parallel to address)        |
| channels   | Vec\<String\> | Channel filter ("all" or specific, e.g. "HZ")  |
| fwd_data   | bool          | Forward raw seismic data packets               |
| fwd_alarms | bool          | Forward ALARM/RESET event messages             |

**Validation**: `address.len() == port.len()` must hold; rejected at startup otherwise.

### ForwardDestination (new)

Runtime representation of a single forwarding target.

| Field          | Type                       | Description                                      |
| -------------- | -------------------------- | ------------------------------------------------ |
| id             | usize                      | Destination index (0, 1, 2, ...)                 |
| addr           | SocketAddr                 | Resolved destination address (IP + port)         |
| socket         | UdpSocket                  | Bound local UDP socket for sending               |
| tx             | mpsc::Sender\<ForwardMsg\> | Channel to send packets to forwarding task       |

**Lifecycle**: Created at startup, lives for duration of application.

### ForwardMsg (new)

Message sent from pipeline to forwarding task.

| Variant | Payload                          | Description                          |
| ------- | -------------------------------- | ------------------------------------ |
| Data    | `Vec<u8>` (raw packet bytes)     | Seismic data packet to forward       |
| Alarm   | `String` (formatted message)     | ALARM/RESET event text               |

### ForwardStats (new)

Per-destination forwarding statistics for periodic logging.

| Field        | Type     | Description                                |
| ------------ | -------- | ------------------------------------------ |
| packets_sent | u64      | Total packets successfully sent            |
| packets_dropped | u64   | Packets dropped due to full queue          |
| send_errors  | u64      | UDP send errors (unreachable, etc.)        |
| last_send_at | Option\<Instant\> | Timestamp of last successful send |

**Reset**: Counters are cumulative (never reset). Periodic log reports delta since last report.

## Relationships

```
ForwardSettings (config)
    │
    ├─ 1:N → ForwardDestination (runtime, one per address+port pair)
    │          │
    │          └─ owns → UdpSocket, ForwardStats
    │
    └─ shared → ForwardMsg channel (pipeline writes, task reads)
```

## State Transitions

### Forwarding Task Lifecycle

```
Startup → Running → Shutdown
   │          │
   │          ├─ receives Data msg → filter by channel → sendto() → update stats
   │          ├─ receives Alarm msg → sendto() → update stats
   │          ├─ stats interval elapsed → log stats summary
   │          └─ send error → increment send_errors, log warning
   │
   └─ config validation fails → Error (application exits)
```

### No Runtime State Changes

Forwarding configuration is static after startup. No hot-reload of destinations or filters.
