# Kernel IPC (Host Model)

`kernel/src/ipc/` (`Message`, `Channel`, `Endpoint`).

- `Message::try_new` bounds payload to `IPC_MAX_PAYLOAD` (DoS bound).
- `Channel` is a bounded queue with Full/Empty/Closed errors; host-only, no blocking/wakeup yet.
- No shared memory; owner checks are future work (S14). Kernel IPC bridges to user-space QEOS Service Bus (`docs/IPC_PROTOCOL.md`).