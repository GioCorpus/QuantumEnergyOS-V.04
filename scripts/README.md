# Scripts

Reproducible local checks (mirror CI):

- `scripts/check.sh`: `cargo fmt --check`, `cargo clippy`, `cargo test`, frontend `tsc`/`eslint`/`build`.
- No hidden warnings; fail on first error.