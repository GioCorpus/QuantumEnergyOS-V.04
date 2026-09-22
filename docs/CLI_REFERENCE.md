# QEOS V.04 — CLI Reference

**Status:** STABLE (as of Phase 7.8)

The `qeos` binary (`crates/qeos-cli`) provides real commands over the platform
crates. Convention: `--help` prints usage, output is structured JSON, exit codes
are 0 (success), 1 (error) and 2 (usage).

## Commands

| Command | Purpose |
|---|---|
| `qeos doctor` | run platform/backend self-checks |
| `qeos node --id ID` | boot a host node and list discovered devices |
| `qeos gpu run --size N` | run a GPU vec_add via the CPU reference and verify it |
| `qeos qpu simulate --shots N --seed S` | run a seeded QPU parity measurement |
| `qeos experiment run --seed S` | run a reproducible research workflow |
| `qeos dataset register --id ID --version N` | register a versioned dataset |

## Example output (abridged)

`qeos qpu simulate --shots 256 --seed 7`:

```json
{
  "backend": "qpu-sim",
  "zeros": 256, "ones": 0,
  "logical_error_rate": 0.0,
  "simulation_only": true
}
```

## Honesty

Every command reports the actual backend. Hardware (`gpu_vendor_available`,
`qpu_vendor_available`) is reported as `false` because no physical/vendor
device is present. No command fabricates hardware or performance results.
