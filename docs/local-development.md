# Local development

## Prerequisites

| Tool | Needed | Why | Without it |
|---|---|---|---|
| Git | Required | Source control (S §7.2) | Nothing works |
| `rustup` | Required | Installs the toolchain in `rust-toolchain.toml` (S §6.1) | Nothing builds |
| `just` | Optional | The command contract (S §16.1) | Run the commands listed in [CONTRIBUTING.md](../CONTRIBUTING.md#commands) |
| `cargo-nextest` | Optional | Faster test runs (S §14.3) | `just test` falls back to `cargo test` |
| `cargo-deny` | Optional locally, required in CI | Dependency policy in `deny.toml` (S §17.2) | Review new dependencies by hand; `just deny` reports the skip |

A profile that adds a requirement adds its row here. For example, the persistence profile adds a
container runtime and a real PostgreSQL, and SQLite is not a substitute (S §2.3). A required
capability that a machine lacks is recorded as an ADR, not worked around (P §19).

Brief §19 records the capability check behind this table. Rerun it on a new machine:

```bash
rustc --version; cargo --version; rustup --version
for t in just docker podman cargo-nextest cargo-deny cargo-audit psql gh; do
  printf '%-16s' "$t"; command -v "$t" >/dev/null 2>&1 && "$t" --version 2>&1 | head -1 || echo 'NOT INSTALLED'
done
```

## First run

1. Clone the repository, and confirm that `main` is checked out.
2. Run `just bootstrap`.
3. Run `just check-fast`.

The basic path needs no production account, cloud login, shared database, or private secret
(S §12.5, §24.1). If it ever does, record that as a defect rather than documenting it as a step.

## Inner loop

1. Rely on editor diagnostics while editing, and run focused tests with `cargo test -p <crate>`.
2. Run `just check-fast` often.
3. Run `just check` before requesting review. CI runs the same recipe (S §16.5).

A domain-only change needs no infrastructure running (S §24.2).

## Offline work

Cargo builds and tests offline after the first dependency fetch (`cargo build --offline`). These still
need the network: the first fetch, installing a toolchain that is not yet present, and the advisory-database update inside
`cargo deny check`.
