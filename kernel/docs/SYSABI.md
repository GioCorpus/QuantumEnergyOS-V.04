# QEOS SYSABI V1

Stable internal syscall ABI. Numbers in `src/syscall/numbers.rs`.

- 0 read, 1 write, 2 open, 3 close
- 10 spawn, 11 exit
- 20 sleep
- 30 send, 31 recv
- 40 map, 41 alloc

Dispatcher: `dispatch_syscall(number, a0, a1, a2)`.
Never exposes internal kernel structs. Linux compat lives in future `linux-compat/` layer, not claimed in V1.
