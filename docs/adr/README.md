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
