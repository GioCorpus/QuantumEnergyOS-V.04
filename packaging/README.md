# Packaging — Reproducible Builds (V.04 §22)

- Every package declares dependencies; no unspecified pnpm/cargo versions in CI.
- Node: `packageManager: pnpm@9.12.0`; Vite React plugin explicitly declared.
- Rust: workspace-pinned deps in root `Cargo.toml`; `Cargo.lock` committed.
- Targets: Linux x86_64 + ARM64.
- Arch-compatible: pacman-compatible metadata for OS packages; prefer integration
  over reinventing glibc/GCC/Binutils/Coreutils/Bash/systemd.