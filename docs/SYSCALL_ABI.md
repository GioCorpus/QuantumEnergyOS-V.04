# QEOS V.04 — Syscall ABI Specification

**Document Version:** 2.0  
**Phase Milestone:** P5-02 — Syscall ABI  
**Status:** ACTIVE SPECIFICATION & IMPLEMENTATION  
**ABI Stability:** Phase 5 Baseline (ABI v1.0)  

---

## 1. Overview & Architectural Principles

The QEOS Syscall ABI establishes the unforgeable binary interface between user-space processes/services and the operating system kernel.

### Architectural Rules:
1. **Zero Raw Kernel Pointers**: Kernel internal virtual addresses are never returned or leaked to user space. Opaque handles (`fd`, `channel_id`, `pid`, `device_handle`) are used exclusively.
2. **Strict User Buffer Validation**: All buffer pointers and lengths supplied by user applications undergo validation via `validate_range`:
   - Checks that buffer base pointer `ptr != 0` when `len > 0`.
   - Checks that address addition `ptr.checked_add(len)` does not overflow `usize`.
   - Checks that `ptr + len <= max_user_address` (within the process address window).
3. **Capability Gating**: Privileged syscalls (device access, telemetry inspection, DMA mapping, system administration) verify that the calling process's `CapSet` contains the requisite capability token.
4. **Structured Error Model**: Syscall errors are mapped to a typed, negative 64-bit integer code (`SyscallError`).

---

## 2. Calling Convention & Register Mapping (x86_64)

| Register | Direction | Purpose |
|---|---|---|
| `RAX` | Input | Syscall Number (`SyscallNo`) |
| `RDI` | Input | Argument 0 (`a0`) — Handle / Resource ID |
| `RSI` | Input | Argument 1 (`a1`) — Buffer Pointer / Flags |
| `RDX` | Input | Argument 2 (`a2`) — Length / Size / Option |
| `R10` | Input | Argument 3 (`a3`) — Extended Parameter |
| `R8`  | Input | Argument 4 (`a4`) — Extended Parameter |
| `R9`  | Input | Argument 5 (`a5`) — Extended Parameter |
| `RAX` | Output | Return Value (positive/zero) or Error Code (negative) |

---

## 3. Syscall Number Registry (`SyscallNo`)

```text
+--------+---------------+--------------------+-------------------------------------------+
| Number | Syscall Name  | Subsystem Domain   | Description                               |
+--------+---------------+--------------------+-------------------------------------------+
| 0      | Read          | VFS / IO           | Read bytes from file handle into user buf |
| 1      | Write         | VFS / IO           | Write bytes from user buf to file handle  |
| 2      | Open          | VFS / IO           | Open inode path with access flags         |
| 3      | Close         | VFS / IO           | Close active file handle                  |
| 4      | Stat          | VFS / IO           | Retrieve metadata for inode or handle     |
| 10     | Spawn         | Process            | Spawn a new child process with TID/PID    |
| 11     | Exit          | Process            | Terminate calling process/thread          |
| 12     | Yield         | Scheduler          | Voluntarily yield remaining CPU timeslice |
| 13     | GetPid        | Process            | Return current process PID                |
| 14     | Wait          | Process            | Wait for child process exit status        |
| 15     | ThreadSpawn   | Thread             | Spawn a new thread within current process |
| 16     | ThreadExit    | Thread             | Terminate calling thread                  |
| 20     | Sleep         | Time               | Put calling thread to sleep for ms duration|
| 21     | TimeNow       | Time               | Query monotonic system timestamp (ns)     |
| 30     | Send          | IPC                | Send message across IPC channel           |
| 31     | Recv          | IPC                | Receive message from IPC channel          |
| 32     | ChannelCreate | IPC                | Create a bounded IPC channel endpoint     |
| 33     | ChannelClose  | IPC                | Close an IPC channel endpoint              |
| 40     | Map           | Memory             | Map virtual memory page                   |
| 41     | Alloc         | Memory             | Allocate heap memory region               |
| 42     | Unmap         | Memory             | Unmap virtual memory page                 |
| 43     | Protect       | Memory             | Modify memory page protection flags       |
| 50     | DeviceOpen    | Driver / Device    | Open hardware device with Capability::DeviceRead |
| 51     | DeviceClose   | Driver / Device    | Close hardware device handle               |
| 52     | DeviceRead    | Driver / Device    | Read device configuration/state            |
| 53     | DeviceWrite   | Driver / Device    | Write device control register (Capability::DeviceWrite) |
| 54     | DeviceIoctl   | Driver / Device    | Execute device-specific control command   |
| 60     | TelemetryRead | Telemetry          | Read energy and performance counters       |
| 61     | TelemetrySample| Telemetry         | Capture instantaneous telemetry sample    |
| 70     | CapCheck      | Security           | Check if current process possesses capability |
| 71     | CapDrop       | Security           | Irrevocably drop process capability       |
+--------+---------------+--------------------+-------------------------------------------+
```

---

## 4. Syscall Error Model (`SyscallError`)

```text
+-------------------+-------+--------------------------------------------------------+
| Error Variant     | Code  | Meaning                                                |
+-------------------+-------+--------------------------------------------------------+
| Unknown           | -1    | Syscall number not recognized in ABI registry          |
| Denied            | -2    | Missing required capability or permission check failed |
| InvalidArg        | -3    | Invalid parameter value or zero allocation size        |
| NotFound          | -4    | Requested handle, inode, PID, or device not found      |
| ResourceExhausted | -5    | System memory full or IPC queue backpressure reached   |
| Timeout           | -6    | Operation timed out before completion                  |
| BufferOverflow    | -7    | Pointer + length exceeds user bounds or overflows usize|
| BadPointer        | -8    | NULL pointer passed for non-zero length buffer         |
+-------------------+-------+--------------------------------------------------------+
```

---

## 5. Security & Validation Sequence

For each invoked syscall:
1. `SyscallNo::try_from(n)` validates that the number is recognized; otherwise returns `SyscallError::Unknown`.
2. Pointer validation: For buffers, `validate_range(ptr, len, max_user_addr)` validates non-nullity, bounds, and absence of integer overflow; returns `SyscallError::BadPointer` or `SyscallError::BufferOverflow`.
3. Privilege verification: For hardware (`Device*`), telemetry (`Telemetry*`), or memory mapping (`Map`/`Protect`), `require(&ctx.caps, required_access)` verifies permissions; returns `SyscallError::Denied`.
4. Resource lookup: File handles and channel endpoints are resolved from process-local tables without exposing kernel addresses.
