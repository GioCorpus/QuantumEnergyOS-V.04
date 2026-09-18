# QEOS V.04 — Syscall ABI Specification

**Document Version:** 1.0  
**Status:** ACTIVE SPECIFICATION  
**ABI Stability:** Phase 5 Baseline  

---

## 1. Overview & Principles

The QEOS Syscall ABI defines the binary interface between user-space applications/services and the kernel. It guarantees strict hardware isolation, unforgeable capability validation, and memory safety.

### Core Rules:
1. **No Raw Kernel Pointers**: Pointers into kernel space are never returned to user space. Resources are represented by opaque 64-bit handles (`FileHandle`, `ChannelId`, `DmaHandle`, `ProcessHandle`).
2. **Strict User Buffer Validation**: All user pointers, buffer lengths, and memory regions must be validated for nullity, address bounds, and integer overflow before any memory dereference.
3. **Capability Enforcement**: All privileged syscalls verify that the calling process possesses the requisite `Capability` in its `CapSet`.

---

## 2. Calling Convention & Register Mapping

On x86_64 targets, syscalls are invoked via the `syscall` instruction:

| Register | Purpose | Direction |
|---|---|---|
| `RAX` | Syscall Number (`SyscallNo`) | Input |
| `RDI` | Argument 0 (`a0`) | Input |
| `RSI` | Argument 1 (`a1`) | Input |
| `RDX` | Argument 2 (`a2`) | Input |
| `R10` | Argument 3 (`a3`) | Input |
| `R8`  | Argument 4 (`a4`) | Input |
| `R9`  | Argument 5 (`a5`) | Input |
| `RAX` | Return Value / Error Code | Output |

---

## 3. Syscall Table (`SyscallNo`)

```text
+--------+---------------+--------------------+-------------------------------------------+
| Number | Syscall Name  | Domain             | Description                               |
+--------+---------------+--------------------+-------------------------------------------+
| 0      | Read          | VFS / IO           | Read bytes from file handle               |
| 1      | Write         | VFS / IO           | Write bytes to file handle                |
| 2      | Open          | VFS / IO           | Open inode with flags                     |
| 3      | Close         | VFS / IO           | Close active file handle                  |
| 10     | Spawn         | Process / Exec     | Spawn a new process with TID/PID          |
| 11     | Exit          | Process / Exec     | Terminate current process / thread        |
| 12     | Yield         | Scheduler          | Yield CPU time slice                      |
| 20     | Sleep         | Time / Timer       | Sleep for specified duration (ms)         |
| 30     | Send          | IPC                | Send message across IPC channel           |
| 31     | Recv          | IPC                | Receive message from IPC channel          |
| 40     | Map           | Memory             | Map virtual memory page                   |
| 41     | Alloc         | Memory             | Allocate heap memory region               |
| 42     | Unmap         | Memory             | Unmap virtual memory page                 |
| 50     | DeviceControl | Hardware / Driver  | Control device via capability handle      |
| 60     | TelemetryRead | Observability      | Read hardware telemetry sample counters   |
+--------+---------------+--------------------+-------------------------------------------+
```

---

## 4. Syscall Errors & Status Codes

All syscall errors return a negative error code or a typed `SyscallError` enum:

| Error Name | Code | Description |
|---|---|---|
| `Unknown` | -1 | Syscall number unrecognized or unsupported |
| `Denied` | -2 | Permission denied / missing capability |
| `InvalidArg` | -3 | Invalid argument, null pointer, or length overflow |
| `NotFound` | -4 | Resource, file handle, or channel not found |
| `ResourceExhausted` | -5 | Memory limit or channel capacity exceeded |
| `Timeout` | -6 | Operation timed out |

---

## 5. Security Validation Protocol

Every syscall handler must execute the following validation sequence before taking action:
1. **Decode & Match**: Validate that `SyscallNo::try_from(n)` succeeds.
2. **Capability Check**: If the syscall accesses hardware, devices, or telemetry, invoke `kernel::syscall::validate::require(&process.caps, required_cap)`.
3. **Pointer Validation**: If buffer arguments are supplied, invoke `kernel::syscall::validate::validate_range(ptr, len, user_max_addr)` to ensure `ptr != 0`, `ptr + len` does not overflow, and the entire slice falls within the user address window.
4. **Handle Lookup**: Resolve opaque handle from the process handle table; return `SyscallError::InvalidArg` or `SyscallError::NotFound` if absent.
