# ADR-0002: Split `lint` into `domain`, `application`, and a thin `apps/lint` binary

- Status: Accepted
- Date: 2026-09-14
- Owners: maintainer (brief §17, "Architecture and technology")
- Work item: `work-items/LINT-001.md`
- Supersedes: none

## Context

LINT-001 is the first crate boundary in the repository — `Cargo.toml`'s `members` is empty and
`docs/architecture.md` §4 says "no crates yet." A crate split is explicitly one of the choices that
needs an ADR (`docs/adr/README.md`; K §11.5), and S §5.1 says to start with few crates and split only at
a real responsibility, dependency, compilation, or reuse boundary — never merely to shorten files
(K §10.2).

S §4's `cli` profile row names the shape directly: "Domain and application crates, and a thin
`apps/<name>` whose argument parsing is an adapter." LINT-001 crosses five conceptual boundaries (brief
§5: CLI adapter, file walker, Markdown parser, rule engine, text reporter), but only two of them —
domain policy (the rules) and application orchestration (walk → read → parse → check → decide exit
code) — have a reuse or testability boundary today: the domain rules must stay free of `pulldown-cmark`
and `clap` types so they can be unit-tested and reused by LINT-002–LINT-004 without pulling in I/O; the
orchestration needs to be testable with fakes (S §14.1) independent of the real file system. The walker,
the reader, the parser adapter, and the CLI argument adapter have exactly one implementation each, no
other crate consumes them, and nothing in brief §5–§9 asks them to be swappable yet.

## Decision

Three crates, wired by the dependency rule (S §3.1: frameworks → application → domain, domain depends on
neither):

- `crates/domain` (package `lint-domain`): value objects (`Location`, `RuleId`, `Finding`), the
  `Document` type with its validating constructor, the `Rule` trait, and the `Md009` / `Md047`
  implementations. No dependency on `pulldown-cmark`, `clap`, `std::fs`, or any I/O type.
- `crates/application` (package `lint-application`): the `Walker`, `SourceReader`, and `MarkdownParser`
  ports; the `LintFault` taxonomy; the `run_lint` use case that composes ports and domain rules; the pure
  `format_finding` and `decide_exit_code` functions. Depends on `lint-domain` only.
- `apps/lint` (binary package `lint`): the composition root (`main.rs`) and every adapter — `StdWalker`,
  `StdSourceReader`, `PulldownMarkdownParser` (wrapping `pulldown-cmark`), and the `clap`-based argument
  parser. Depends on `lint-application` and `lint-domain`.

No separate adapter crates. The walker, reader, parser adapter, and CLI adapter live as modules inside
`apps/lint` because nothing outside `apps/lint` consumes them yet (S §5.1) — creating crates for them now
would be a split with no responsibility, dependency, compilation, or reuse boundary behind it.

Per the `Cargo.toml` template's own instruction ("List every crate in `members` explicitly, and only
once it exists"), this ADR and the architecture contract fix the intended layout; `rust-implementer`
creates the crate directories, their `Cargo.toml` files, and adds their paths to the root `members` list
in the same change that adds their source.

## Options considered

| Option | For | Against | Evidence |
|---|---|---|---|
| Three crates: `domain`, `application`, `apps/lint` (chosen) | Matches S §4's `cli` profile shape exactly; keeps `domain` free of technology types so rules stay unit-testable and reusable by LINT-002–004; lets application logic be tested with fakes (S §14.1) without a real file system; smallest split that gives both properties | `apps/lint` is a wider "adapters" bucket than a fully layered design would use | S §4 `cli` row; S §5.1 "start with few crates" |
| One crate (no split) | Simplest possible `Cargo.toml` | Domain rules would compile against `pulldown-cmark`/`clap`/`std::fs` directly, so a "domain unit test" could never be free of I/O and framework types (S §3.1, §5.2); contradicts the dependency rule this project has committed to in `docs/architecture.md` §3 | `docs/architecture.md` §3; S §3.1 |
| Four-plus crates (separate `crates/walker`, `crates/parser-adapter`, `crates/cli` adapters) | Maximal separation, ready if a second binary ever needs the same adapters | No second consumer exists; speculative split violates K §10.2 ("split at a real … boundary," "never merely to shorten files") and S §3.5's ban on unneeded portability | K §10.2; S §3.5 |
| Defer (no crate ADR yet, let the implementer decide) | N/A | LINT-001 is the first card; leaving the layout to the implementer with no contract means Development starts from an unreviewed design, and `docs/architecture.md` §4/§5/§6 explicitly wait on this card | `docs/architecture.md` §4–§6 |

## Consequences

- Easier: `crates/domain` can be unit-tested and property-tested with no I/O; `crates/application`'s
  `run_lint` can be tested with in-memory fakes for `Walker`/`SourceReader`/`MarkdownParser`, giving a
  cheap layer below the integration tests already required by every AC's "Verified by" column.
- Harder: adapters (`walker.rs`, `reader.rs`, `parser.rs`, `cli.rs` inside `apps/lint`) can only be
  exercised through `apps/lint`'s own tests (unit tests within the binary crate, or the `tests/cli_*.rs`
  integration tests against the built binary) until a real second consumer justifies splitting them out.
- Newly constrained: `crates/domain` may never add a dependency on `pulldown-cmark`, `clap`, `std::fs`,
  or `std::env` — a lint-worthy invariant, though this card adds no automated enforcement of it (see
  "Left unverified" in the architecture contract).
- Left unverified: whether this three-crate shape still fits once LINT-002–LINT-004 add ten more rules
  and LINT-005 adds configuration-driven rule selection. Revisit if a rule needs a port `domain` cannot
  express without an I/O or technology dependency.

## Follow-up

`rust-implementer` creates the three crate directories and their manifests in the LINT-001 candidate
commit, and updates the root `Cargo.toml` `members` and `docs/architecture.md` §4's crate table in the
same commit. Revisit this ADR when LINT-002–LINT-008 either confirm the split holds or force a fourth
crate at a real boundary.
