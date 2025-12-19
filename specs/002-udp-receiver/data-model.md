# Data Model: UDP Packet Reception

## Entities

### `Packet`

Represents a single received UDP datagram. This is the raw data unit ingested by the system.

| Field | Type | Description |
|---|---|---|
| `source` | `std::net::SocketAddr` | The IP address and port of the sender. |
| `data` | `Vec<u8>` | The raw payload bytes of the packet. |
| `received_at` | `std::time::Instant` | Timestamp of reception (for latency tracking). |

## Queue Structure

### `PacketChannel`

An asynchronous Multi-Producer Single-Consumer (MPSC) channel used to pass `Packet` entities from the receiver task to the processing task.

- **Sender**: Held by the `UdpListener` task.
- **Receiver**: Held by the `Processor` task (stubbed for now).
- **Capacity**: Configurable (e.g., 1024 packets) to handle burst traffic.
