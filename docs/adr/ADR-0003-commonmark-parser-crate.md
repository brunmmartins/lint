# ADR-0003: Use `pulldown-cmark` as the CommonMark parser

- Status: Accepted
- Date: 2026-09-14
- Owners: maintainer (brief §17, "Architecture and technology")
- Work item: `work-items/LINT-001.md`
- Supersedes: none

## Context

Brief §6 requires a CommonMark parser, chosen at the architecture stage, free to use, and clearing
`deny.toml`'s licence allow-list (`Apache-2.0`, `MIT`, `Unicode-3.0`). Brief §4's abuse cases include
"very long lines, deeply nested lists or block quotes, huge files" causing "a panic, a hang, or unbounded
memory" — the parser's own robustness under hostile input is a fitness criterion, not just its API.

Checked 2026-09-14 against crates.io's API (`https://crates.io/api/v1/crates/pulldown-cmark`) and
docs.rs (`https://docs.rs/crate/pulldown-cmark/latest`), cross-checked against the upstream GitHub
repository (`https://github.com/pulldown-cmark/pulldown-cmark`):

- Current version: `0.13.4`. Licence: `MIT` (clears `deny.toml`'s allow-list; `MIT` is already allowed).
- API: an iterator of `Event`s over a `&str`, with no AST allocation forced on the caller — a caller can
  drive it to completion to validate structure without retaining every event, which fits this card's
  scope (below).
- The crate's own documentation states its parser does not use recursion for nested block structure,
  specifically to avoid stack-overflow on deeply nested input — directly relevant to brief §4's "deeply
  nested lists or block quotes" abuse case.
- It is the CommonMark parser used by `rustdoc` and the Rust Playground, which is evidence of wide
  production use and low `unsafe` risk tolerance from that consumer, though this is not a substitute for
  reading its own advisory history.
- No open RUSTSEC advisory is asserted here from memory; `rust-implementer` must run `just deny` once the
  dependency is added, per `rust-supply-chain-security`, rather than rely on this ADR's snapshot.

Changeability check (S §20.5), since this becomes a dependency of `apps/lint`'s adapter layer, not of
`domain`:

1. Is the type part of the business language? No — `pulldown_cmark::Event` is a parsing-library type; the
   business language is "a Markdown document has lines, headings, code fences, etc.," which `domain`
   expresses with its own `Document` type (ADR-0002).
2. Does it force a runtime, serialization, storage, or protocol choice? No runtime or storage forcing;
   it is a synchronous, in-process, pure function over `&str`.
3. What changes if the crate is replaced? Only `apps/lint`'s `parser.rs` adapter (ADR-0002) — the single
   module that constructs a `domain::Document` from source text.
4. Is a small conversion at the boundary enough? Yes — this card's `Document` only needs line text and
   trailing-newline structure; the adapter drives the parser for its structural-validity/robustness
   property, without yet projecting the full event stream into `domain` (left for whichever of
   LINT-002–LINT-004 needs heading/block structure).
5. Is the abstraction cost greater than the realistic cost of replacing it? No — one adapter module, one
   call site (`apps/lint`'s composition root); replacing the parser touches no other crate.

## Decision

Use `pulldown-cmark` `0.13.4` as the CommonMark parser, depended on only by `apps/lint` (never by
`crates/domain` or `crates/application`, per ADR-0002's dependency rule). `rust-implementer` pins this
exact version in `apps/lint`'s `Cargo.toml` (or `[workspace.dependencies]`, ADR-0002) and runs `just deny`
before the candidate is handed to Review.

## Options considered

| Option | For | Against | Evidence |
|---|---|---|---|
| `pulldown-cmark` (chosen) | MIT-licensed, clears `deny.toml`; non-recursive block parsing (fits brief §4's nested-input abuse case); iterator API with no forced AST allocation; widely used (rustdoc) | Its CommonMark conformance has known, documented edge-case deviations (as any CommonMark implementation does) — acceptable, since brief §3 explicitly disclaims "byte-for-byte parity with markdownlint" and this card needs "parses without panicking," not full spec conformance testing | crates.io API, docs.rs, upstream GitHub, checked 2026-09-14 |
| `comrak` | Also MIT/BSD-licensed, GFM-flavored, has an AST API which would suit later structural rules (headings, lists) more directly than event-only `pulldown-cmark` | Larger dependency surface (more transitive crates, an AST allocated up front) for a card whose only two rules (MD009, MD047) are line-based, not AST-based (S §20.5 Q2/Q5: the extra abstraction cost is not justified yet); heavier default footprint than this card's ACs need | crates.io, checked 2026-09-14 |
| `markdown-it` (Rust port) | Plugin-based, close to the reference JS `markdown-it` | Smaller, less battle-tested Rust ecosystem footprint than `pulldown-cmark`; no clear advantage for this card's line-based rules | crates.io, checked 2026-09-14 |
| Hand-write only what MD009/MD047 need (no CommonMark parser at all) | Zero new dependency for this card, since both rules are actually line-based | Contradicts brief §5's outcome, which frames the card as crossing "the Markdown parser" boundary and states it "sets the extension points LINT-002 through LINT-008 build on" — those cards need real CommonMark structure (headings, fences, lists); deferring parser selection would leave that boundary undesigned | Brief §5 |
| Defer (pick no parser yet, let LINT-002 decide) | N/A | Brief §6 assigns this choice to `rust-solution-architect` for LINT-001 specifically, and `docs/architecture.md` §4/§6 are waiting on it | Brief §6; `docs/architecture.md` §4, §6 |

## Consequences

- Easier: future structural rules (MD001, MD018, MD022, MD025, MD031, MD032, MD040) have a maintained,
  safety-conscious parser already wired into `apps/lint`'s composition root.
- Harder: nothing for this card — MD009/MD047 do not consume the event stream's structure, only the raw
  line text `domain::Document` extracts from the source.
- Newly constrained: `crates/domain` and `crates/application` must never import `pulldown_cmark` types
  directly (ADR-0002); any future card that wants event/AST structure in `domain` must add an explicit
  translation type there, not re-export `pulldown_cmark::Event`.
- Left unverified: `just deny`'s actual advisory/licence check has not been run in this task (no crate
  exists yet to run it against). `rust-implementer` must run it once the dependency is added, and treat a
  skip (missing `cargo-deny` locally) as "not a pass" per `rust-supply-chain-security`.

## Follow-up

`rust-implementer` runs `just deny` after adding the dependency and reports the result in the candidate's
checks. Revisit this ADR if `just deny` surfaces an advisory, or when a later card's structural rules
reveal the event-stream translation this card deliberately deferred.
