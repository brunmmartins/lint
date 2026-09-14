# Architecture decision records

Each consequential decision gets its own file, `ADR-NNNN-<slug>.md`, numbered in sequence and started
from [TEMPLATE.md](TEMPLATE.md) (K §11.5). Add the index row in the same commit as the ADR.

Write an ADR when a future maintainer would otherwise reopen the argument. Typical subjects are the stack
profile, a crate split, the async runtime, the persistence model, a protocol, the compatibility policy,
an unsafe boundary, a major dependency, a missing required capability (P §19), and any exception to a K
policy (P §1.1).

- **Never rewrite an accepted ADR.** A changed decision is a new ADR that supersedes the old one. The old
  one gains a `Superseded by` line, its index row changes status, and nothing else in it changes.
- **The brief and ADRs hold different things.** The brief records what is true now. An ADR records what
  was decided, what was rejected, and on what grounds.
- **A decision that spans projects** is recorded by the project that owns it, and linked from the other.

## Index

| ADR | Decision | Status | Date | Work item |
|---|---|---|---|---|
| [ADR-0001](ADR-0001-cli-stack-profile.md) | Adopt only the `cli` stack profile for `lint` | Accepted | 2026-09-14 | `LINT-001` |
| [ADR-0002](ADR-0002-crate-layout.md) | Split into `domain`, `application`, and a thin `apps/lint` binary | Accepted | 2026-09-14 | `LINT-001` |
| [ADR-0003](ADR-0003-commonmark-parser-crate.md) | Use `pulldown-cmark` 0.13.4 as the CommonMark parser | Accepted | 2026-09-14 | `LINT-001` |
| [ADR-0004](ADR-0004-argument-parser-crate.md) | Use `clap` 4.6.6 (derive) as the argument parser | Accepted | 2026-09-14 | `LINT-001` |
| [ADR-0005](ADR-0005-walker-no-symlink-follow.md) | Walk directories without following symlinks; no walker crate | Accepted | 2026-09-14 | `LINT-001` |
| [ADR-0006](ADR-0006-exit-code-precedence-and-output-order.md) | I/O faults take exit-code precedence over findings; fixed output order | Accepted | 2026-09-14 | `LINT-001` |
