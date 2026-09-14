# ADR-0004: Use `clap` (derive API) as the command-line argument parser

- Status: Accepted
- Date: 2026-09-14
- Owners: maintainer (brief §17, "Architecture and technology")
- Work item: `work-items/LINT-001.md`
- Supersedes: none

## Context

Brief §6 requires an argument parser, chosen at the architecture stage, free to use, clearing
`deny.toml`. AC6 requires: given no path argument, print a usage message to stderr and exit 2. Brief §6
also fixes the exit-code contract project-wide: 0/1/2 with 2 reserved for "usage, configuration, or I/O
error." A parser whose own default failure behavior already matches "usage message to stderr, exit 2"
removes a whole class of hand-written error-handling code that would otherwise have to reimplement it.

Checked 2026-09-14 against crates.io's API (`https://crates.io/api/v1/crates/clap`) and docs.rs
(`https://docs.rs/crate/clap/latest`), cross-checked against the upstream GitHub repository
(`https://github.com/clap-rs/clap`):

- Current version: `4.6.6`. Licence: `MIT OR Apache-2.0` (both already allowed in `deny.toml`).
- Default behavior on a parse error (including a missing required positional argument) is to print a
  usage message to stderr and call `std::process::exit(2)` — clap's own documented default exit code for
  a usage error is 2, which is exactly AC6's contract with no custom code.
- The `derive` feature lets the CLI surface (`lint <path>...`) be declared as a struct, keeping the
  argument adapter (S §4 `cli` row: "a thin `apps/<name>` whose argument parsing is an adapter") small
  and declarative rather than hand-parsing `std::env::args()`.

Changeability check (S §20.5): `clap` is used only inside `apps/lint`'s `cli.rs` adapter module (ADR-0002
already excludes it from `domain`/`application`). If replaced, only that module changes; the parsed
result is translated at the boundary into an application-level `Vec<PathBuf>` before anything else sees
it, so no `clap` type crosses into `domain` or `application`.

## Decision

Use `clap` `4.6.6` with the `derive` feature, depended on only by `apps/lint`. Define one required,
repeatable positional argument (`paths: Vec<PathBuf>`, `num_args = 1..`) so that omitting it triggers
clap's own usage-error path (stderr message, exit 2) and satisfies AC6 without bespoke error handling.
`rust-implementer` pins this exact version and runs `just deny` before the candidate is handed to Review.

## Options considered

| Option | For | Against | Evidence |
|---|---|---|---|
| `clap` with `derive` (chosen) | MIT/Apache-2.0, clears `deny.toml`; default usage-error behavior already matches AC6's exit-2 contract; declarative derive keeps the CLI adapter thin; the de facto standard, so behavior is well-documented and predictable for future cards (LINT-005's config flags, LINT-007's `--json`, LINT-008's `--fix`) | Larger dependency graph than a minimal parser; brings in `clap_builder`, `clap_derive`, and their own transitive crates | crates.io, docs.rs, upstream GitHub, checked 2026-09-14 |
| `lexopt` / hand-rolled `std::env::args()` parsing | Minimal dependency footprint, full control over exact messages | AC6's usage message and exit-2 behavior, plus every later flag LINT-005/007/008 will add, would have to be hand-written and kept consistent by hand; no derive ergonomics; higher long-term maintenance cost for a CLI contract brief §7 calls a compatibility surface | K §1.1 favors simplicity, but here "simple" means fewer lines of hand-maintained parsing/error code, which `clap` gives, not fewer dependencies |
| `pico-args` | Very small, zero-dependency | No built-in usage-message generation or structured exit-code convention; AC6 would need hand-written usage text and exit(2) call, reopening exactly what `clap` already solves | crates.io, checked 2026-09-14 |
| Defer (let `rust-implementer` pick) | N/A | Brief §6 assigns this choice to `rust-solution-architect` for LINT-001; the CLI shape is a compatibility surface (brief §7) that should not be improvised during Development | Brief §6; brief §7 |

## Consequences

- Easier: AC6 is satisfied by clap's own default behavior, not new code; later cards' flags
  (`--config`, `--json`, `--fix`) extend the same derive struct.
- Harder: nothing identified for this card.
- Newly constrained: the derive struct's field names, positional arity, and help text become part of the
  CLI compatibility surface (brief §7) the moment `apps/lint` ships; changing them later is a breaking
  change like any other part of the CLI contract.
- Left unverified: clap's exact usage-message wording is not fixed by this ADR or the contract — AC6 only
  requires "a usage message," not specific text, so the implementer's derive-generated text is acceptable
  as long as it goes to stderr with exit 2.

## Follow-up

`rust-implementer` runs `just deny` after adding the dependency and confirms AC6's exit-2 behavior via
`tests/cli_no_path_given.rs`. Revisit this ADR only if a future card's flag needs cannot be expressed by
clap's derive API, or if `just deny` surfaces an advisory.
