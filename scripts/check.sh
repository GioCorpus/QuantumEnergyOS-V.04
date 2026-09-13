#!/usr/bin/env bash
set -euo pipefail
echo "=== QuantumEnergyOS V.04 reproducible checks ==="
echo "--- cargo fmt --check ---"
cargo fmt --all -- --check
echo "--- cargo clippy ---"
cargo clippy --all-targets --all-features -- -D warnings
echo "--- cargo test (Mock/Simulator only, no physical QPU) ---"
cargo test --workspace
echo "--- python tests ---"
python -m pytest tools -q
echo "--- frontend (if deps installed) ---"
if [ -d dashboard/node_modules ]; then
  (cd dashboard && pnpm exec tsc --noEmit && pnpm exec eslint src --max-warnings 0 && pnpm build)
else
  echo "dashboard/node_modules missing; CI runs frontend build"
fi
echo "=== ALL CHECKS PASSED ==="