# Tools — Development Toolchain (V.04 §21)

Rust (Cargo/rustup), GCC/G++, Binutils, Make, CMake, Clang/LLVM, GDB, Bash,
Python 3.12, Node 20 + pnpm 9.12.

Quantum dev: Microsoft Quantum Development Kit / Q# + Python quantum tooling
(current supported versions only).

Python helpers here must include `requirements.txt` and `pytest` tests; CI runs
`python -m pytest tools -q`.