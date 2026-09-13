# QEOS Kernel V1

Rust-first monolithic modular kernel core. See `docs/SYSABI.md`, `docs/debugging.md`.

```bash
cargo fmt --check
cargo check -p qeos-kernel
cargo clippy -p qeos-kernel -- -D warnings
cargo test -p qeos-kernel
cargo run -p qeos-kernel
```
