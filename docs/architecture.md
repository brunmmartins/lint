# Architecture

This describes how lint is put together **now**. The reasons for its shape are ADRs, indexed
in [adr/README.md](adr/README.md). If this document changes shape, an ADR changes with it.

## 1. Context

Not yet; needed by `rust-solution-architect` for LINT-001.

## 2. Stack profile

Profiles in use: `cli` (brief §6). The decision record is not yet written; it is ADR-0001, written by
`rust-solution-architect` for LINT-001. That ADR also
lists the profiles not chosen and what would add each one. Capabilities are additive, and none is adopted
speculatively (S §3.5, §4; P §6).

## 3. Dependency rule

Frameworks and infrastructure depend on application contracts. Application contracts depend on the
domain. The domain depends on neither (S §3.1).

- Compile-time dependencies point inward only. Ports are declared by the application, and adapters
  implement them.
- Only a composition root knows every concrete technology. It loads configuration, constructs adapters,
  and wires use cases (S §3.3, §5.5).
- Port errors are the application's own categories. They never wrap a library's error type (S §5.3).
- Transport, storage, and domain representations stay separate, with explicit conversion at each
  boundary (S §20.3–20.4).

## 4. Crates

No crates yet. The first crates, and their rows here, arrive with LINT-001.

Every crate in this table is listed in `members` in the root `Cargo.toml`, and every member is in this
table. Split a crate only at a real responsibility, dependency, compilation, or reuse boundary (K §10.2).

## 5. Composition roots

Not yet; needed by `rust-solution-architect` for LINT-001.

## 6. Ports, adapters, and replacement boundaries

Not yet; needed by `rust-solution-architect` for LINT-001.

## 7. Configuration

Not yet. The configuration file arrives with LINT-005 (brief §12). No environment variables or secrets.
