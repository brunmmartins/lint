# LINT-001: Walking skeleton — walk paths, parse Markdown, run MD009 and MD047, report text findings, exit correctly

> Evidence record. The board is authoritative for state, priority, WIP, owners, and timestamps.
> Everything above **Log** describes the item, and is edited as understanding improves.
> The log is append-only.

## Identity

- Card: `LINT-001` on the board
- Demand type: capability
- Shaping decision: accept — brief §5 already scopes this as the first delivering slice (walk paths,
  parse, engine, MD009, MD047, text output, exit codes), and it is small enough for one implementation
  pass. No clarification, combination, rejection, redirection, split, or discovery is needed.
- Class of service: standard (no fixed date, no incident)
- Customer or affected system: a person at a local shell, and a CI job, both reading `lint`'s stdout,
  stderr, and exit status (brief §4)
- Brief sections this item draws on: §3, §4, §5, §6, §7, §8, §9, §10

## Outcome

A person writing Markdown, or a CI job, runs `lint <path>...` and gets a walking skeleton of the full
tool: paths and directories are walked for Markdown files, each file is parsed, the rule engine runs
MD009 (`no-trailing-spaces`) and MD047 (`single-trailing-newline`) against it, and every finding is
printed as `path:line:col RULE message`. The exit status tells a human or a CI job, without reading
output, whether problems were found or the invocation itself failed. This crosses the CLI adapter, the
file walker, the Markdown parser, the rule engine, and the text reporter, and sets the extension points
LINT-002 through LINT-008 build on (brief §5). It matters because it is the first observable behavior of
`lint` and the first CLI/output/exit-code contract the project commits to (brief §6, §7).

## Context

- **Current behavior:** none — no code exists yet in this worktree. `lint` does not run at all today.
- **Evidence for the problem:** brief §3, "problem today" — Markdown style problems such as trailing
  whitespace and a missing final newline are today found by eye in review, or with a tool that needs a
  Node.js runtime. This is the requester's statement, not a measurement.
- **Constraints:**
  - Output format is fixed by brief §3/§6: `path:line:col RULE message`, one finding per line, on
    stdout; errors, warnings, and usage go to stderr.
  - Exit-code contract is fixed by brief §6: 0 when no finding remains, 1 when at least one finding
    remains, 2 on a usage, configuration, or I/O error.
  - Only MD009 and MD047 are in scope for this card; the other ten rules in brief §3's twelve-rule set
    are excluded (brief §5).
  - No configuration file, no inline comments, no JSON output, no `--fix` in this card (brief §5).
  - Untrusted input: file contents, directory trees, and paths are untrusted (brief §4, §15). No input
    may cause a panic, a hang, or unbounded recursion — including symlink loops.
  - CLI contract, once set here, is a compatibility surface (brief §7): changing it later is a breaking
    change.

## Acceptance criteria

| ID | Given / When / Then | Verified by |
|---|---|---|
| AC1 | Given a single path to one Markdown file with no MD009/MD047 problems, When `lint` runs with that path, Then it prints nothing to stdout and exits 0. | `tests/cli_walk_single_file.rs` (integration test against the built binary) |
| AC2 | Given a single path to one Markdown file containing a line with trailing spaces, When `lint` runs with that path, Then it prints one line to stdout in the form `path:line:col MD009 message` for that finding and exits 1. | `tests/cli_md009.rs` |
| AC3 | Given a single path to one Markdown file missing a single trailing newline (or with more than one), When `lint` runs with that path, Then it prints one line to stdout in the form `path:line:col MD047 message` and exits 1. | `tests/cli_md047.rs` |
| AC4 | Given a directory containing Markdown files nested in subdirectories, some clean and some with MD009/MD047 problems, When `lint` runs with that directory as the path, Then it walks it recursively, reports a finding line for every problem found in every nested file, and exits 1 if any finding was reported, 0 otherwise. | `tests/cli_walk_directory.rs` |
| AC5 | Given multiple paths on the command line that mix a clean file, a dirty file, and a directory, When `lint` runs with all of them, Then findings from every path are reported (order is stable and documented), and the exit code is 1 if any path produced a finding, 0 only if none did. | `tests/cli_multiple_paths_mixed.rs` |
| AC6 | Given no path argument, When `lint` runs, Then it prints a usage message to stderr and exits 2. | `tests/cli_no_path_given.rs` |
| AC7 | Given a path that does not exist on disk, When `lint` runs with that path, Then it prints an error naming the path to stderr and exits 2, without crashing and without silently skipping the path. | `tests/cli_nonexistent_path.rs` |
| AC8 | Given a path to a file that exists but cannot be read (e.g. permission denied), When `lint` runs with that path, Then it prints an error naming the path to stderr and exits 2, without a panic. | `tests/cli_unreadable_file.rs` |
| AC9 | Given a directory path that contains no Markdown files, When `lint` runs with that path, Then it prints nothing to stdout, prints no error, and exits 0 (an empty walk is not itself an error). | `tests/cli_walk_no_markdown_files.rs` |
| AC10 | Given a Markdown file whose trailing-space line is also missing the final newline, When `lint` runs, Then both MD009 and MD047 findings are reported for that file with correct, independent line/column locations, and the exit code is 1. | `tests/cli_md009_and_md047_same_file.rs` |
| AC11 | Given any of the above inputs, When `lint` runs, Then it never panics and never hangs, including on a directory tree containing a symbolic-link loop (brief §4, "abuse cases"). | `tests/cli_symlink_loop_does_not_hang.rs` |

## Non-goals

- Every rule other than MD009 and MD047 (MD001, MD010, MD012, MD013, MD018, MD022, MD025, MD031, MD032,
  MD040) — each is its own card (LINT-002, LINT-003, LINT-004).
- The configuration file that enables/disables rules or sets rule options — LINT-005.
- Inline disable/enable comments inside Markdown files — LINT-006.
- JSON output — LINT-007.
- `--fix` / automatic fixing — LINT-008.
- Byte-for-byte parity with markdownlint's output, configuration format, or edge-case behavior (brief §3).
- Any network access, subprocesses, or environment-variable-driven configuration (brief §9).

## Risks and dependencies

- **Security and privacy:** file contents, directory trees, and paths are untrusted input (brief §4,
  §15). The walker and parser must not panic, hang, or use unbounded memory on hostile input — very
  long lines, deeply nested structure, huge files, invalid UTF-8, or a symlink loop (brief §4). `lint`
  must echo only locations, rule IDs, and messages — never file contents — into its output (brief §8).
  This card performs no writes (no `--fix` yet), so file-integrity risk is out of scope here.
- **Compatibility and migration:** none yet to migrate from (no prior release). This card establishes
  the CLI, output, and exit-code contract that brief §7 calls a compatibility surface for every card
  after it — an escalation trigger in itself, since it fixes "a public API, a contract, or a stored data
  shape".
- **External dependencies:** a CommonMark/Markdown parser crate and a command-line argument parser
  crate. Which crates, and the crate layout, are not yet decided — that is owned by
  `rust-solution-architect` for LINT-001 (brief §6, §20) and must clear `deny.toml`'s licence policy
  (brief §18).
- **Operational impact:** none. `lint` is a local CLI with no deployment, no service, no on-call (brief
  §11, §14).

## Validation approach

- **Automated tests:** unit tests for the MD009 and MD047 rule logic in the rule engine; integration
  tests against the built binary (per brief §7's "checked by integration tests against the built
  binary") covering AC1–AC11 above, under `tests/`.
- **Manual or environment verification:** run the built `lint` binary over this repository's own `docs/`
  directory and confirm the output matches expectations by eye (brief §10, post-release verification).
- **Performance or security validation:** none beyond dependency vetting (`cargo-deny`, brief §18) at
  the architecture stage. No performance budget is measured yet (brief §13 records an assumption, not a
  target, for this card).

## Flow and delivery plan

- **Delivery point:** per brief §10 — the change reaching GitHub's `main` at
  github.com/brunmmartins/lint is delivery; local integration into `main` is not delivery. The
  repository does not exist yet; the maintainer creates it and decides every push. Not yet delivered.
- **Rollout:** N/A — no released version yet (brief §10, §11: no feature flags, no release cadence).
- **Rollback or forward fix:** N/A until delivered; once delivered, `git revert` of the commit on `main`
  per brief §11.
- **Post-release verification:** per brief §10 — from a clean checkout of the verified commit on
  `main`, `CI=true just check` passes and `lint` run over the repository's own `docs/` exits as this
  card's acceptance criteria predict.

## Architecture contract

- Author and date: rust-solution-architect, 2026-09-14
- Route: full
- Brief sections relied on: §3, §4, §5, §6, §7, §8, §9, §13, §15
- ADRs: ADR-0001 (Proposed), ADR-0002 (Proposed), ADR-0003 (Proposed), ADR-0004 (Proposed), ADR-0005
  (Proposed), ADR-0006 (Proposed)

### Boundaries and placement

| Acceptance criterion | Boundaries crossed | Crate or module | New or existing |
|---|---|---|---|
| AC1 | CLI adapter → application → domain (parse, run rules, no findings) | `apps/lint::cli`, `apps/lint::walker`, `apps/lint::reader`, `apps/lint::parser`, `lint-application::run_lint`, `lint-domain::rules::{md009,md047}` | new |
| AC2 | CLI adapter → application → domain (MD009 finding) → text formatting | same as AC1, plus `lint-application::format_finding` | new |
| AC3 | CLI adapter → application → domain (MD047 finding) → text formatting | same as AC1, plus `lint-application::format_finding` | new |
| AC4 | CLI adapter → walker (recursive directory discovery) → application → domain | `apps/lint::walker::StdWalker`, `lint-application::run_lint` | new |
| AC5 | CLI adapter → application (multi-path aggregation, ordering) → domain | `apps/lint::main`, `lint-application::run_lint`, `lint-application::decide_exit_code` | new |
| AC6 | CLI adapter only (clap's own usage-error path) | `apps/lint::cli` | new |
| AC7 | CLI adapter → walker (not-found fault) → application (fault → exit code) | `apps/lint::walker::StdWalker`, `lint-application::LintFault`, `lint-application::decide_exit_code` | new |
| AC8 | CLI adapter → reader (unreadable fault) → application (fault → exit code) | `apps/lint::reader::StdSourceReader`, `lint-application::LintFault` | new |
| AC9 | CLI adapter → walker (empty directory, no fault, no finding) → application | `apps/lint::walker::StdWalker`, `lint-application::run_lint` | new |
| AC10 | CLI adapter → application → domain (two independent rules over one `Document`) | `lint-domain::Document`, `lint-domain::rules::{md009,md047}` | new |
| AC11 | walker (no symlink-follow, iterative traversal) → reader (size bound) | `apps/lint::walker::StdWalker` (ADR-0005), `apps/lint::reader::StdSourceReader` | new |

Everything in this table is new: LINT-001 is the first card and the first crates in the repository.

### Crates and dependency direction

- Crates touched, and the boundary each represents:
  - `crates/domain` (package `lint-domain`) — domain policy: value objects, the `Document` type, the
    `Rule` trait and its MD009/MD047 implementations. No I/O, no framework types.
  - `crates/application` (package `lint-application`) — use-case orchestration: the `Walker`,
    `SourceReader`, and `MarkdownParser` ports; the `LintFault` taxonomy; `run_lint`; the pure
    `format_finding` and `decide_exit_code` functions.
  - `apps/lint` (binary package `lint`) — composition root and every adapter: `cli.rs` (`clap`),
    `walker.rs`, `reader.rs`, `parser.rs` (`pulldown-cmark`), `main.rs`.
- New crates, and why a module is not enough (K §10.2): see ADR-0002. In short — `domain` must stay free
  of `pulldown-cmark`/`clap`/`std::fs` so its rules are unit-testable and reusable by LINT-002–LINT-004;
  `application` must be testable with fakes (S §14.1) independent of the real file system; a single
  module inside one crate could not enforce either separation at compile time.
- Allowed dependencies: `lint-domain` → none (not even `std::fs`, `std::env`, or a parsing crate);
  `lint-application` → `lint-domain` only; `apps/lint` → `lint-application`, `lint-domain`,
  `pulldown-cmark`, `clap`. No dependency points outward from `lint-domain` or `lint-application`.
- Dependencies added, with version, licence, the source checked, and the S §20.5 answers: see ADR-0003
  (`pulldown-cmark` 0.13.4, MIT) and ADR-0004 (`clap` 4.6.6 with the `derive` feature, MIT OR Apache-2.0).
  Both checked against crates.io's API and docs.rs on 2026-09-14; both clear `deny.toml`'s licence
  allow-list without a new exception. The `[workspace.dependencies]` entries below pin these exact
  versions so `rust-implementer` does not re-decide them.

### Ports

| Port | Operation signatures | Send and dispatch choice | Error type and variants | Transaction or ordering needs |
|---|---|---|---|---|
| `Walker` (`lint-application`) | `fn resolve(&self, input: &Path) -> Result<Vec<PathBuf>, WalkFault>` — resolves one CLI input (file or directory) into zero or more file paths to lint | Static dispatch (generic `impl Walker` / `W: Walker`), plain sync `fn` — no async anywhere in this card (K §14.1: start synchronous; nothing here needs concurrency), so no `Send` bound question applies | `WalkFault { NotFound { path }, Unreadable { path, detail } }` — application's own taxonomy (K §13.1: not-found, permanent), never a raw `io::Error` | Files within one directory are returned in sorted-by-name order (ADR-0005); the caller (`run_lint`) preserves the order of the `inputs` slice across calls |
| `SourceReader` (`lint-application`) | `fn read(&self, path: &Path) -> Result<String, ReadFault>` — reads and UTF-8-validates one file's bytes, enforcing the size bound before allocating | Static dispatch, plain sync `fn` | `ReadFault { Unreadable { path, detail }, TooLarge { path, limit_bytes }, InvalidUtf8 { path } }` (K §13.1: permanent, exhausted, invariant respectively) | None — one file, one read, no transaction |
| `MarkdownParser` (`lint-application`) | `fn parse(&self, source: &str) -> lint_domain::Document` — drives a full CommonMark parse for its robustness/validity property, then builds `Document` from the source's lines | Static dispatch, plain sync `fn`, infallible for this card's scope (`pulldown-cmark` does not error on malformed Markdown; ADR-0003) | n/a — infallible | None |
| `Rule` (`lint-domain`) | `fn check(&self, doc: &Document) -> Vec<Finding>` — a pure, deterministic policy over one parsed document | `dyn Rule` in a static registry (`pub const RULES: &[&dyn Rule]`) — the engine already iterates a heterogeneous, extensible list of rules today (MD009, MD047), a real requirement now, not a speculative one, so runtime-style dispatch is justified per S's "use `dyn` only when runtime selection is a real requirement" | Infallible — a rule is a pure function from `Document` to `Vec<Finding>`; it has no failure mode of its own | Rules run independently in registry order; findings from all rules for one file are merged and sorted by `(line, column)` before printing (ADR-0006) |

### Data and translation

- Types introduced, their invariants, and their validating constructors:
  - `lint_domain::Location { line: NonZeroUsize, col: NonZeroUsize }` — 1-based, matching the
    `path:line:col` output format; constructed only through `Location::new(line, col)`, which is the only
    place a zero value is rejected.
  - `lint_domain::RuleId { Md009, Md047 }` — closed enum, `Display` prints `"MD009"` / `"MD047"`.
  - `lint_domain::Finding { rule: RuleId, location: Location, message: String }`.
  - `lint_domain::Document` — private fields; constructed only via `Document::from_source(source: &str)`,
    which splits `source` into 1-indexed lines (text with the line terminator stripped) and records the
    count of consecutive trailing newline characters at end-of-file (0 = no trailing newline at all, 1 =
    exactly one, 2+ = more than one) for MD047 to use directly, without re-scanning.
  - `lint_application::LintFault` — unifies `WalkFault`/`ReadFault` into one enum the composition root
    reports on stderr and folds into the exit-code decision (ADR-0006): `NotFound`, `Unreadable`,
    `TooLarge`, `InvalidUtf8`, each carrying the offending `path` and a human-readable `detail`.
- Translations at each boundary:
  - CLI args (`Vec<OsString>` via `clap`) → `apps/lint::cli::Cli { paths: Vec<PathBuf> }` (clap adapter).
  - Raw bytes (`std::fs::read`) → UTF-8-validated `String` (`StdSourceReader`; `ReadFault::InvalidUtf8` on
    failure, `ReadFault::TooLarge` before the read even happens, checked via `std::fs::metadata` first —
    "validation happens before allocation," K §22.2).
  - `String` → `lint_domain::Document` (`PulldownMarkdownParser`; drives `pulldown_cmark::Parser` to
    completion for its safety/validity property, per ADR-0003, then builds `Document` from the source's
    own lines — the parsed `Event` stream is not yet retained as domain structure in this card).
  - `WalkFault` / `ReadFault` → `lint_application::LintFault` (explicit `From` conversions inside
    `lint-application`, so `apps/lint` never has to match on two different fault enums).
  - `Finding` → one formatted text line (`lint_application::format_finding`, the fixed
    `path:line:col RULE message` shape) → stdout (`apps/lint::main`, the only place that writes stdout).
- Stored or emitted shapes, with the contract direction and compatibility promise (brief §7): the only
  emitted shape is the stdout text line format brief §3/§6 fixes (`path:line:col RULE message`) and the
  three-value exit code (brief §6, precedence fixed by ADR-0006). Both are hand-maintained, checked by
  the `tests/cli_*.rs` integration tests against the built binary (brief §7). No stored shape — `lint`
  owns no durable state (brief §8).
- Migration steps (expand and contract), if any: not applicable — no prior shape exists to migrate from
  (first card, brief §10).

### Runtime behavior

- Configuration keys (prefix and separator from brief §12), with defaults and validation: not
  applicable — no configuration file in this card (brief §12 defers it to LINT-005); no environment
  variables (brief §9).
- Concurrency model, the blocking-work strategy, cancellation safety: synchronous, single-threaded,
  no async runtime (K §14.1: start synchronous; nothing in AC1–AC11 needs concurrency). Each input path
  is processed to completion before the next. No cancellation surface exists (no signals handled beyond
  the OS default; a `Ctrl-C` simply terminates the process, which is acceptable for a short-lived local
  CLI with no partial writes to protect in this card — `--fix`'s write-safety is LINT-008's concern).
- Bounds on input size, depth, time, allocation, and queues, with the overload behavior:
  - File size: `MAX_FILE_BYTES = 10 MiB` (`apps/lint::reader`), checked via `std::fs::metadata` before
    any read, so an oversized file is never allocated. Overload behavior: `ReadFault::TooLarge`, reported
    on stderr, contributes to exit 2 (ADR-0006), the file is skipped (not partially linted).
  - Directory depth: unbounded by design, but safely so — traversal is iterative (an explicit
    `Vec<PathBuf>` stack in `StdWalker`, never recursive function calls), so depth cannot exhaust the
    native call stack (ADR-0005). Combined with "never follow a symlink as a directory" (ADR-0005), a
    cycle cannot form, so no separate cycle-count bound is needed either.
  - Line length: no separate bound. `Document::from_source` and each `Rule::check` process every line
    with a single linear scan, so one pathologically long line costs time proportional to its own length
    once; the file-size bound already caps total work per file.
  - Findings: bounded by input size (one Finding per line-level problem at most); no separate cap.
- Timeouts, deadlines, retries, and idempotency for each external call: not applicable — no network
  calls, no subprocesses (brief §9); file-system calls are local, synchronous, and not retried (a local
  I/O fault is treated as permanent for this invocation, per `ReadFault`/`WalkFault`'s taxonomy).
- Observability: spans and fields, metrics and their labels, domain outcomes: not applicable — brief §14
  states no observability beyond stdout findings, stderr errors, and the exit status; this card adds no
  tracing or metrics.
- Startup and shutdown effects: `main` returns `std::process::ExitCode` (never calls
  `std::process::exit` directly), so stdout/stderr are flushed through Rust's normal `main`-return path
  before the process exits (K §24.3, "exit meaningfully"). No other startup/shutdown effects — no
  connections to open or drain.

### Security

- Trust boundaries crossed, and where authorization is enforced (K §22.2): the process reads whatever
  its own OS file permissions allow (brief §4 — "the process has the user's own file permissions"); no
  additional authorization layer. File contents, directory trees, and paths are untrusted content per
  brief §4/§15, even though the process invoking `lint` is trusted.
- Untrusted inputs, with their bounds and validation points:
  - Path strings (argv, via `clap`): no length bound beyond the OS's own argv limits; validated by
    attempting to resolve them (`StdWalker`), with `WalkFault::NotFound` for a nonexistent path.
  - File bytes: bounded to `MAX_FILE_BYTES` (above), validated as UTF-8 before any further processing.
  - Directory structure: symlinks never followed as directories (ADR-0005), ruling out loops by
    construction rather than merely bounding them.
  - Markdown content: parsed once through `pulldown-cmark`, chosen in part for its non-recursive block
    parsing (ADR-0003), which addresses brief §4's "deeply nested lists or block quotes" abuse case at
    the parser level, not just the walker level.
- Secrets and redaction: not applicable — `lint` has no secrets in its threat model (brief §12, §15).
- Abuse cases the tests must cover: hostile Markdown (long lines, deep nesting, huge files, invalid
  UTF-8), hostile directory trees (symlink loops), from brief §4. AC11 covers the symlink-loop case
  directly. The oversized-file and invalid-UTF-8 cases are not named by their own AC ID, but fall within
  AC8's "exists but cannot be read [as valid input]" intent — `rust-implementer` extends the same
  `ReadFault` family and adapter test coverage to `TooLarge` and `InvalidUtf8`, not only permission
  denial, per this contract's Test plan below.

### Test plan

| Acceptance criterion | Layer | Test (planned path::name) | Real dependency or fake | Negative cases |
|---|---|---|---|---|
| AC1 | Adapter/system (binary) | `tests/cli_walk_single_file.rs` | Real filesystem (tempdir) | n/a (positive case) |
| AC2 | Domain unit + adapter/system | `crates/domain/src/rules/md009.rs` unit tests; `tests/cli_md009.rs` | Domain: no fake needed (pure). System: real filesystem | Line with only leading/internal spaces (no finding); line with trailing tab |
| AC3 | Domain unit + adapter/system | `crates/domain/src/rules/md047.rs` unit tests; `tests/cli_md047.rs` | Domain: pure. System: real filesystem | Exactly one trailing newline (no finding); empty file |
| AC4 | Application (fakes) + adapter/system | `crates/application/src/lib.rs` (or `tests/`) unit test for nested aggregation with fake `Walker`/`SourceReader`/`MarkdownParser`; `tests/cli_walk_directory.rs` | Application layer: fakes. System: real filesystem | Directory with only clean nested files (exit 0) |
| AC5 | Application (fakes, ordering) + adapter/system | Application unit test asserting output order = input order then sorted walk order; `tests/cli_multiple_paths_mixed.rs` | Application: fakes. System: real filesystem | All-clean mix (exit 0); all-dirty mix |
| AC6 | Adapter/system | `tests/cli_no_path_given.rs` | Real binary invocation (relies on clap's default usage-error behavior, ADR-0004) | n/a |
| AC7 | Adapter unit + adapter/system | `apps/lint::walker` unit test for `WalkFault::NotFound` mapping (tempdir); `tests/cli_nonexistent_path.rs` | Adapter: real filesystem (tempdir), no fake. System: real filesystem | Nonexistent path mixed with a valid path (still processes the valid one) |
| AC8 | Adapter unit + adapter/system | `apps/lint::reader` unit test for `ReadFault::Unreadable` (unix permission bits); `tests/cli_unreadable_file.rs` | Adapter/system: real filesystem, unix permission bits | Flag: unreliable when the test runner is root (permission checks are bypassed) — left unverified in that environment, see below |
| AC9 | Adapter unit + adapter/system | `apps/lint::walker` unit test for a directory with only non-Markdown files → empty result, no fault; `tests/cli_walk_no_markdown_files.rs` | Real filesystem (tempdir) | Directory containing only non-`.md`/`.markdown` files |
| AC10 | Domain unit + adapter/system | `crates/domain` unit test: one `Document` with both a trailing-space line and a missing final newline, asserting both `Finding`s with independent correct locations; `tests/cli_md009_and_md047_same_file.rs` | Domain: pure. System: real filesystem | n/a (both-at-once is itself the case) |
| AC11 | Adapter unit + adapter/system | `apps/lint::walker` unit test asserting a symlink-to-directory is never recursed into (property, not just an example); `tests/cli_symlink_loop_does_not_hang.rs` (unix-only, `#[cfg(unix)]`, asserts process termination within a bounded wall-clock timeout) | Real filesystem, `std::os::unix::fs::symlink` | n/a — non-termination is the failure mode under test |

- Property, fuzz, performance, or resilience checks, and which stage runs them: no property/fuzz test is
  mandated by this contract (it would need a new dev-only dependency, e.g. `proptest`, which is out of
  scope to vet for this card). Recommended, not required: a property test that `Document::from_source`
  applied to `s + "\n"` always yields `trailing_newline_count == 1` regardless of `s`'s trailing-newline
  state, as a good MD047 regression guard. Left for `rust-implementer`'s discretion or a future card.

### Constraints for the implementer

- Must: keep `crates/domain` free of `pulldown-cmark`, `clap`, `std::fs`, `std::env`, and any other I/O
  or framework type (ADR-0002); enforce the `MAX_FILE_BYTES` bound before allocating a read buffer, not
  after (K §22.2); use `std::fs::symlink_metadata`, never plain `metadata`, before deciding whether to
  recurse into a directory entry (ADR-0005); return `std::process::ExitCode` from `main`, not call
  `std::process::exit` directly; pin `pulldown-cmark = "0.13.4"` and `clap = "4.6.6"` (features =
  `["derive"]`) exactly as vetted in ADR-0003/ADR-0004, and run `just deny` before handing off the
  candidate.
- Must not: let any `WalkFault`/`ReadFault`/`io::Error` cross into `lint-domain` or appear in a
  `lint-domain` type's public signature; let a single unreadable/nonexistent/oversized path abort
  processing of the other paths given on the command line (every AC7/AC8-style fault must still let
  sibling paths be linted, per AC5's "findings from every path are reported"); introduce a fourth exit
  code value or change the 0/1/2 meanings (ADR-0006, brief §6).
- Stop and ask if: `just deny` surfaces an advisory or licence conflict for `pulldown-cmark` or `clap`
  that this contract's 2026-09-14 vetting did not anticipate (an ADR change, not a silent substitution);
  the exact CommonMark event-stream-to-domain-structure translation seems to be needed already for
  MD009/MD047 (it should not be — if it looks necessary, the domain model chosen here is probably wrong
  and the architect should be re-engaged rather than improvised around).

### Left unverified

- Whether `crates/domain` staying free of I/O/framework types is actually enforced, verified by:
  `rust-quality-reviewer` reading the crate's `Cargo.toml` dependency list and `use` statements at Review.
- Whether `just deny` passes with the exact pinned versions, verified by: `rust-implementer` running it
  and reporting the result; `rust-quality-reviewer` re-checking it was run.
- AC8's permission-denied behavior when the check/build/test runner has root privileges (permission bits
  are bypassed for root on most platforms), verified by: whoever runs `just check` noting the runner's
  effective UID; not verifiable in this contract without knowing the CI/build environment's user.
- The mixed "fault on one path, finding on another" exit-code precedence ADR-0006 fixes, since no AC
  literally exercises it end-to-end, verified by: `rust-implementer` adding a test beyond the eleven ACs'
  minimum, or failing that, `rust-validation-engineer` confirming `decide_exit_code` is at least
  unit-tested for that combination.
- Windows/non-unix behavior of the walker and the symlink-loop test (`#[cfg(unix)]`), verified by:
  nobody in this card — brief §11's target build/check platform is Linux aarch64; out of scope until a
  non-goal is reversed.
- Whether the three-crate split still fits once LINT-002–LINT-004 add ten more rules and LINT-005 adds
  configuration-driven rule selection, verified by: those cards' own architecture contracts.

## Decisions and ADRs

| ADR | Decision | Status |
|---|---|---|
| [ADR-0001](../docs/adr/ADR-0001-cli-stack-profile.md) | Adopt only the `cli` stack profile for `lint` | Proposed |
| [ADR-0002](../docs/adr/ADR-0002-crate-layout.md) | Split into `domain`, `application`, and a thin `apps/lint` binary | Proposed |
| [ADR-0003](../docs/adr/ADR-0003-commonmark-parser-crate.md) | Use `pulldown-cmark` 0.13.4 as the CommonMark parser | Proposed |
| [ADR-0004](../docs/adr/ADR-0004-argument-parser-crate.md) | Use `clap` 4.6.6 (derive) as the argument parser | Proposed |
| [ADR-0005](../docs/adr/ADR-0005-walker-no-symlink-follow.md) | Walk directories without following symlinks; no walker crate | Proposed |
| [ADR-0006](../docs/adr/ADR-0006-exit-code-precedence-and-output-order.md) | I/O faults take exit-code precedence over findings; fixed output order | Proposed |

All six ADRs are `Proposed`. Per brief §17, "Architecture and technology, including ADR acceptance" is
decided by the maintainer, and "development may proceed on Proposed ADRs (instruction of 2026-09-14)" —
so `rust-implementer` may start from this contract without waiting for acceptance, but the maintainer
still owns accepting or rejecting each ADR.

## Log

### 2026-09-14: LINT-001 shaped from Options into a Ready-candidate record (intake)

- Event: handoff
- Candidate: not-yet-built
- Checks run: none (intake stage, no code)
- Checks recommended but not run: none
- Evidence: work-items/LINT-001.md created; docs/application-brief.md §3, §5, §6 read
- Next: rust-solution-architect, write the architecture contract and Proposed ADRs for LINT-001

### 2026-09-14: Architecture contract and six Proposed ADRs written for LINT-001 (rust-solution-architect)

- Event: handoff
- Candidate: not-yet-built
- Checks run: `cargo metadata --no-deps --format-version 1` → exit 0 (confirms the still-empty workspace
  manifest parses; no crate source or member paths were added, per the manifest's own instruction to
  list a member "only once it exists")
- Checks recommended but not run: `just deny` — no dependency exists yet to check; `rust-implementer`
  runs it after adding `pulldown-cmark` and `clap` per ADR-0003/ADR-0004
- Evidence: `work-items/LINT-001.md` "Architecture contract" section (maps AC1–AC11 to crates/ports,
  fixes ports/errors/dispatch, data and translation, runtime bounds, security, test plan, implementer
  constraints, left-unverified list) and "Decisions and ADRs" table; `docs/adr/ADR-0001-cli-stack-profile.md`
  through `docs/adr/ADR-0006-exit-code-precedence-and-output-order.md` (all `Proposed`), indexed in
  `docs/adr/README.md`; `docs/architecture.md` §1, §2, §3 (dependency-rule detail), §4, §5, §6 updated
  from "Not yet" to the real crate layout, composition root, and ports/adapters table. Crate/version/
  licence facts for `pulldown-cmark` 0.13.4 (MIT) and `clap` 4.6.6 (MIT OR Apache-2.0) checked 2026-09-14
  against crates.io's API and docs.rs, cited in ADR-0003 and ADR-0004. Root `Cargo.toml` left untouched
  (`members = []`): the manifest's own comment says a crate is listed "only once it exists," so
  `rust-implementer` adds the three crate paths when it creates them.
- Next: rust-implementer, build the LINT-001 candidate per the architecture contract and the six
  Proposed ADRs — create `crates/domain`, `crates/application`, `apps/lint`, add them to the root
  `Cargo.toml` `members`, pin `pulldown-cmark = "0.13.4"` and `clap = "4.6.6"` (features = ["derive"]),
  and run `just deny` before handoff.

### 2026-09-14: LINT-001 candidate built — walk, parse, MD009/MD047, text output, exit codes (rust-implementer)

- Event: candidate change
- Candidate: `58141f945bf71ee1655e2dde1323721b0f671cc3`
- Checks run: `CARGO_BUILD_JOBS=4 just check-fast` → exit 0 (fmt check, `cargo check --workspace --all-targets`, 36 lib tests passed); `CARGO_BUILD_JOBS=4 cargo test --workspace --all-features --all-targets` → exit 0, all unit and `tests/cli_*.rs` integration tests passed (0 failed); `CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0, no warnings; `CARGO_BUILD_JOBS=4 cargo test --workspace --doc --all-features` → exit 0, 0 doctests, 0 failed; `CARGO_BUILD_JOBS=4 RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` → exit 0, no warnings; `CARGO_BUILD_JOBS=4 cargo deny check` → "advisories ok, bans ok, licenses ok, sources ok"; `CARGO_BUILD_JOBS=4 just check` → exit 0, re-run directly on the clean committed tree at the candidate commit.
- Checks recommended but not run: `CI=true just check` (adds `--locked`; not run in this task); manual verification of `lint` over this repository's own `docs/` directory (work item's Validation approach; left for Validation stage).
- Evidence: created `crates/domain` (`lint-domain`: `Location`, `RuleId`, `Finding`, `Document::from_source`, the `Rule` trait, `Md009`/`Md047`, `RULES` registry — no I/O/framework dependency, `crates/domain/Cargo.toml` has an empty `[dependencies]` table) and `crates/application` (`lint-application`: `Walker`/`SourceReader`/`MarkdownParser` ports, `WalkFault`/`ReadFault`/`LintFault`, `run_lint`, `format_finding`, `decide_exit_code` — depends on `lint-domain` only); created `apps/lint` (binary `lint`: `main.rs` composition root, `cli.rs` via `clap` 4.6.6 derive, `walker.rs` (`StdWalker`, iterative, never follows a symlink as a directory), `reader.rs` (`StdSourceReader`, 10 MiB bound checked via `std::fs::metadata` before reading), `parser.rs` (`PulldownMarkdownParser` wrapping `pulldown-cmark` 0.13.4)); added all three crate paths to root `Cargo.toml` `members` and pinned `pulldown-cmark`/`clap` in `[workspace.dependencies]`, consumed only by `apps/lint`; updated `CHANGELOG.md` (Unreleased entry) and `docs/architecture.md` §4 (crate table now reflects the crates as created, not "not yet created"). Tests, by acceptance criterion: AC1 `apps/lint/tests/cli_walk_single_file.rs`; AC2 `apps/lint/tests/cli_md009.rs` + `crates/domain/src/rules/md009.rs` unit tests; AC3 `apps/lint/tests/cli_md047.rs` + `crates/domain/src/rules/md047.rs` unit tests; AC4 `apps/lint/tests/cli_walk_directory.rs` + `crates/application/src/run_lint.rs` fake-based unit tests; AC5 `apps/lint/tests/cli_multiple_paths_mixed.rs` + application ordering unit tests; AC6 `apps/lint/tests/cli_no_path_given.rs`; AC7 `apps/lint/tests/cli_nonexistent_path.rs` + `apps/lint/src/walker.rs` unit tests; AC8 `apps/lint/tests/cli_unreadable_file.rs` + `apps/lint/src/reader.rs` unit tests (root-runner case explicitly left unverified per the contract); AC9 `apps/lint/tests/cli_walk_no_markdown_files.rs` + walker unit test; AC10 `apps/lint/tests/cli_md009_and_md047_same_file.rs`; AC11 `apps/lint/tests/cli_symlink_loop_does_not_hang.rs` (bounded 10s wall-clock timeout) + `apps/lint/src/walker.rs` symlink-loop unit test. Also added, beyond the eleven ACs' minimum per ADR-0006's own "Left unverified" note: `crates/application/src/format.rs`'s `fault_takes_precedence_over_a_finding_present_in_the_same_run` test, covering the mixed fault-plus-finding exit-code precedence no single AC exercises end-to-end.
- Next: rust-quality-reviewer, review candidate `58141f945bf71ee1655e2dde1323721b0f671cc3` against the architecture contract, the six ADRs, and `rust-implementation`'s self-review checklist.

### 2026-09-14: LINT-001 candidate reviewed — fail, one blocking finding on test determinism (rust-quality-reviewer)

- Event: gate result: fail
- Candidate: `58141f945bf71ee1655e2dde1323721b0f671cc3`
- Checks run: `git rev-parse` / `git merge-base --is-ancestor` / `git show --stat 405ddc8` / `git diff 58141f9..405ddc8` → candidate confirmed, 405ddc8 append-only to work-items/LINT-001.md's Log; `git merge-base main 58141f9` → `0c8864e4f272cd3dca7663abe95ab43f5287e3ae`, full diff and `git log --oneline` reviewed; `CARGO_BUILD_JOBS=4 cargo test --workspace --all-features --all-targets` → run 9 times: 8 green, 1 FAILED (`reader::tests::rejects_invalid_utf8` panicked at apps/lint/src/reader.rs:78:9, exit 101), passes reliably in isolation — a test-fixture race from pid+nanos temp-directory naming, not a production-code defect; `CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0, no warnings; `CARGO_BUILD_JOBS=4 cargo deny check` → "advisories ok, bans ok, licenses ok, sources ok", exit 0; `CARGO_BUILD_JOBS=4 RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` → exit 0, no warnings; Cargo.lock grepped to confirm pulldown-cmark 0.13.4 and clap 4.6.6 pinned exactly as ADR-0003/ADR-0004 require.
- Checks recommended but not run: `CI=true just check` (--locked) — not run in this task.
- Findings: F1 [blocking] apps/lint/src/reader.rs:116-128, apps/lint/src/walker.rs:101-113, apps/lint/tests/support/mod.rs:24-36 — temp-directory uniqueness relies on pid+nanos, which raced and produced a real (if intermittent) test failure on this exact candidate during this review; tests must be deterministic, replace with tempfile::tempdir() or an atomic counter. F2 [suggestion] apps/lint/src/walker.rs:28-35 — a symlink-to-directory given directly on the command line falls through to being treated as a single file to read, an edge case ADR-0005's text doesn't explicitly reason about; no AC exercises it, not blocking. F3 [nit] crates/domain/src/rules/md047.rs:19-24,37 — defensive `.max(1)`/fallback code that is unreachable given Document::from_source's own invariant; harmless.
- Contract/ADR conformance: ADR-0001 through ADR-0006 all held; crates/domain has zero dependencies (Cargo.toml [dependencies] empty), crates/application depends on lint-domain only, apps/lint is the only crate depending on pulldown-cmark/clap; no library-error type crosses into lint-domain's public signatures; walker never calls plain metadata before recursing, only symlink_metadata; decide_exit_code gives fault strict precedence over findings, unit- and integration-tested for the mixed case.
- AC traceability: all 11 ACs have a real, non-vacuous verifying test (test bodies read in full for AC1-AC11's primary tests plus negative-path tests for AC5/AC7); each asserts exact stdout/exit-code content that would fail if the underlying behavior regressed.
- Next: rust-implementer, fix F1 (nondeterministic temp-directory naming in apps/lint/src/reader.rs, apps/lint/src/walker.rs, and apps/lint/tests/support/mod.rs) on a new candidate commit; F2 and F3 travel with the item as non-blocking (F2 may be addressed at the implementer's discretion, F3 may be left as-is). Human maintainer review and approval is still required before any push to main regardless of this agent review's outcome (docs/git-workflow.md).

### 2026-09-14: LINT-001 candidate reworked — fixed F1 test-fixture nondeterminism (rust-implementer)

- Event: candidate change
- Candidate: `0e5cbcdaee31b398a460ff7bd6878022dc658dce`
- Checks run: `CARGO_BUILD_JOBS=4 just check-fast` → exit 0 (fmt check, `cargo check --workspace --all-targets`, 13 lint-application + 23 lint-domain lib tests passed); `CARGO_BUILD_JOBS=4 cargo test --workspace --all-features --all-targets` → run 12 times in a row, exit 0 every run, 0 FAILED every run, 69 tests passed each run including `reader::tests::rejects_invalid_utf8`, no flakiness reproduced; `CARGO_BUILD_JOBS=4 just check` → exit 0 (fmt clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; full workspace test suite — unit tests plus all 12 `tests/cli_*.rs` integration binaries plus doc-tests — all passed; `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` clean; `cargo deny check` → "advisories ok, bans ok, licenses ok, sources ok").
- Checks recommended but not run: `CI=true just check` (--locked) — not run in this task.
- Evidence: this candidate supersedes `58141f945bf71ee1655e2dde1323721b0f671cc3` (superseded because production/test code changed) and fixes review finding F1 (nondeterministic temp-directory naming). Changed `apps/lint/src/reader.rs:116-131`, `apps/lint/src/walker.rs:101-116`, `apps/lint/tests/support/mod.rs:22-40`: each of the three hand-rolled `tempdir()`/`temp_dir()` helpers now folds a process-local `static AtomicU64` counter (`fetch_add(1, Ordering::Relaxed)`) into the directory name alongside the existing pid+nanos scheme, so two calls racing on the same clock tick within one test binary can never collide — chosen over adding `tempfile` as a dev-dependency because 19 existing call sites across `apps/lint/tests/cli_*.rs` already manage the returned `PathBuf` with explicit `std::fs::remove_dir_all` cleanup, and switching representations would have meant scope-creeping into all of them for this rework. No production behavior changed; F2 and F3 (non-blocking) left as-is per the review's own discretion note.
- Next: rust-quality-reviewer, re-review the new candidate

### 2026-09-14: LINT-001 candidate re-reviewed — pass, F1 confirmed fixed (rust-quality-reviewer)

- Event: gate result: pass
- Candidate: `0e5cbcdaee31b398a460ff7bd6878022dc658dce`
- Checks run: `git rev-parse 0e5cbcd` / `git merge-base --is-ancestor 0e5cbcd HEAD` → confirmed on branch; `git diff 0e5cbcd..69ecf34 --stat` → touches only work-items/LINT-001.md, append-only; `git diff 58141f9..0e5cbcd --stat` → scoped to exactly apps/lint/src/reader.rs, apps/lint/src/walker.rs, apps/lint/tests/support/mod.rs (plus the log entry) — Cargo.toml/Cargo.lock and all crate manifests unchanged, confirming no `tempfile` dependency was added, matching the implementer's report; read all three diffs in full — each of the three hand-rolled tempdir helpers gained a function-local `static AtomicU64` with a genuine `fetch_add(1, Ordering::Relaxed)` folded into the directory name string; grepped every call site (`apps/lint/tests/cli_*.rs` via `support::temp_dir`, and the unit-test callers in `reader.rs`/`walker.rs`) and confirmed all of them route through these three helpers — no path bypasses the counter; `Ordering::Relaxed` is correct here since only the counter's own uniqueness matters, not synchronization with any other memory; `CARGO_BUILD_JOBS=4 cargo test --workspace --all-features --all-targets` → run 15 times in a row (more than the implementer's 12), exit 0 and 0 FAILED every run, race not reproduced; `CARGO_BUILD_JOBS=4 cargo clippy --workspace --all-targets --all-features -- -D warnings` → exit 0, no warnings; `CARGO_BUILD_JOBS=4 cargo deny check` → "advisories ok, bans ok, licenses ok, sources ok".
- Checks recommended but not run: `CI=true just check` (--locked) — not run in this task.
- Findings: none. Re-confirmed F2 (apps/lint/src/walker.rs:28-35) and F3 (crates/domain/src/rules/md047.rs:19-24,37) are untouched by this diff — both left as-is per the prior review's discretion note, as the implementer reported; neither is re-litigated here.
- Contract/ADR conformance: unaffected — this diff touches only test-fixture helper functions, no production behavior, no port/adapter signatures, no ADR-relevant code.
- Findings summary: race condition (F1) confirmed closed; fix is properly scoped, atomic, and sufficient; no regressions found in clippy, deny, or 15 repeated full test-suite runs.
- Next: rust-validation-engineer, verify AC1-AC11 on candidate 0e5cbcdaee31b398a460ff7bd6878022dc658dce. Human maintainer review and approval is still required before any push to main regardless of this agent review's outcome (docs/git-workflow.md).

### 2026-09-14: LINT-001 candidate validated — pass, all eleven ACs verified (rust-validation-engineer)

- Event: gate result: pass
- Candidate: `0e5cbcdaee31b398a460ff7bd6878022dc658dce`
- Checks run: `git worktree add --detach .worktrees/validate-LINT-001 0e5cbcdaee31b398a460ff7bd6878022dc658dce` → succeeded, `git rev-parse HEAD` confirmed exactly `0e5cbcd...`, `git status --short` empty; `CARGO_BUILD_JOBS=4 just check` run twice → exit 0 both times, identical counts (13 lint-application + 23 lint-domain + 13 apps/lint unit tests, all 12 `tests/cli_*.rs` integration binaries, 0 doctests, clippy `-D warnings` clean, rustdoc clean, `cargo deny check` → "advisories ok, bans ok, licenses ok, sources ok"); `CARGO_BUILD_JOBS=4 CI=true just check` → exit 0, only difference from local mode was the expected `--locked` flag on clippy/test/doc, no other divergence, no defect; each of the 11 `tests/cli_*.rs` binaries run individually via `cargo test -p lint --test <name>` → all exit 0, every sub-test `ok`; `cargo test --workspace --all-features --all-targets` run 5 further times → exit 0, 0 failed every run, no flakiness reproduced (adds to the implementer's 12 and reviewer's 15 prior runs); `CARGO_BUILD_JOBS=4 cargo build --release -p lint` → exit 0; `./target/release/lint ../../docs` (this repository's own docs/ directory, read-only) → no output, exit 0, cross-checked independently (no trailing-space lines via `grep -rlP ' +$'`, every `.md` file ends in exactly one `\n`); direct negative-path binary invocations: no args → usage to stderr, exit 2; nonexistent path → error naming the path to stderr, exit 2; directory of only non-Markdown files → silent, exit 0; directory containing a symlink loop → returns immediately, exit 0, well inside a 10s bound.
- Checks recommended but not run: formal performance validation, because brief §13 records no performance budget for this card (assumption, not a target); timeout/shutdown/restart resilience, because `lint` is a short-lived local CLI process with no service lifecycle (brief §14); property/fuzz testing, because the architecture contract leaves it discretionary and out of scope for this card; AC8's permission-denied behavior under a non-root runner, because this validation environment runs as root (uid 0), which bypasses Unix permission bits — this is the architecture contract's own pre-documented "Left unverified" item, confirmed still applicable by directly observing that `chmod 000` is bypassed for root against the built binary.
- Evidence: acceptance-criteria matrix — AC1-AC7, AC9, AC10, AC11 verified with a passing, individually-run test plus, for AC6/AC7/AC9/AC11, a direct invocation of the release binary confirming exact stdout/stderr/exit-code behavior. AC8's test passed but its assertion is vacuous under root, matching its own documented guard and the architecture contract's "Left unverified" note rather than constituting a new gap. No difference found between `just check` and `CI=true just check`. No flakiness observed across 5 additional full-suite runs. Manual verification of `lint` over `docs/` matches the acceptance criteria's predicted clean-exit behavior, independently cross-checked with `grep`/byte inspection.
- Next: rust-release-steward, assess readiness for LINT-001 candidate 0e5cbcdaee31b398a460ff7bd6878022dc658dce — note this needs maintainer go/no-go, and the GitHub repository does not exist yet (brief §10).

### 2026-09-14: LINT-001 release readiness assessed — maintainer decisions and delivery actions pending (rust-release-steward)

- Event: handoff
- Candidate: `0e5cbcdaee31b398a460ff7bd6878022dc658dce`
- Checks run: `python3 .delivery/checkpoint.py status` gave exit 0, confirming the active LINT-001 release-readiness checkpoint at clean head `f3e882c85b751a7a0b1cd276cc61190376055a0c`; candidate existence and ancestry checks gave exit 0; the product-content diff from candidate to branch head was empty and the only later changed path was `work-items/LINT-001.md`; branch status was clean; `main` remained `0c8864e4f272cd3dca7663abe95ab43f5287e3ae`; `git remote -v` returned no remotes; `id -u` returned `0`.
- Checks recommended but not run: AC8 under a non-root account, because uid 0 bypasses Unix permission bits; `CARGO_BUILD_JOBS=4 CI=true just check` on the future squash-integration commit, because integration has not occurred; clean-checkout `CARGO_BUILD_JOBS=4 CI=true just check` and `lint docs/` against GitHub `main`, because the repository has not been created or pushed.
- Evidence: candidate `0e5cbcdaee31b398a460ff7bd6878022dc658dce` is immutable and an ancestor of the branch head; the 2026-09-14 re-review entry records pass; the 2026-09-14 validation entry records `CARGO_BUILD_JOBS=4 CI=true just check` exit 0 on the exact candidate, all eleven test binaries passing, release-build success, and `lint` over `docs/` exiting 0, while explicitly retaining AC8’s non-root limitation. `CHANGELOG.md` contains the LINT-001 user-visible change under Unreleased. Package version remains `0.1.0`, appropriate for the first unpublished CLI with no prior compatibility promise. No database, schema, configuration, migration, specialist-review, signing, or provenance requirement applies. Diagnostics and exit status are the required observability.
- Release plan: locally squash `feature/LINT-001-walking-skeleton` into `main` after the readiness lines close; run `CARGO_BUILD_JOBS=4 CI=true just check` on the exact integration commit. Version stays `0.1.0`; no version tag is planned because brief §10 defines delivery as a GitHub `main` push and states there are no versioned releases. Roll out with one push to `main`. No data steps apply. Roll back with `git revert <full-integration-commit-id>` followed by `git push origin main`; there is no migration cutoff after which rollback stops working. The maintainer creates `github.com/brunmmartins/lint`, reviews the integrated commit, runs `git remote add origin git@github.com:brunmmartins/lint.git`, and runs `git push -u origin main`. From a clean checkout of GitHub `main`, run `CARGO_BUILD_JOBS=4 CI=true just check` and `CARGO_BUILD_JOBS=4 cargo run --release -p lint -- docs`; require exit 0 and no findings. The maintainer observes GitHub’s push event and declares delivery.
- Open readiness lines: ADR-0001 through ADR-0006 remain Proposed; AC8 lacks non-root evidence; maintainer review and release go are absent; local integration, exact-integration CI, GitHub repository creation and push, delivery observation, and post-release verification have not occurred.
- Next: maintainer, accept or reject ADR-0001 through ADR-0006 and give no-go or conditional go after non-root AC8 evidence; orchestrator must treat recorded ADR-status changes as a new candidate and repeat review/validation before integration.

### 2026-09-14: ADR-0001 through ADR-0006 accepted by maintainer (rust-solution-architect)

- Event: handoff
- Candidate: `0e5cbcdaee31b398a460ff7bd6878022dc658dce`
- Checks run: `git diff --check` gave exit 0 with no output; narrow status and diff audit showed only the six ADR status lines and their six `docs/adr/README.md` index cells changed from `Proposed` to `Accepted`
- Checks recommended but not run: `CARGO_BUILD_JOBS=4 CI=true just check`, because the ADR status changes must first be committed as a new candidate and then independently reviewed and validated
- Evidence: the maintainer explicitly accepted ADR-0001 through ADR-0006; `docs/adr/ADR-0001-cli-stack-profile.md` through `docs/adr/ADR-0006-exit-code-precedence-and-output-order.md` and their `docs/adr/README.md` index rows now record `Accepted`, with no architecture-content or code changes. Because these changes are outside the work-item Log, candidate `0e5cbcdaee31b398a460ff7bd6878022dc658dce` is superseded.
- Next: orchestrator, commit the seven ADR artifacts as a new candidate, record its full commit ID, then dispatch renewed review and validation before integration

### 2026-09-14: LINT-001 superseding candidate reviewed — fail, malformed traceability trailer (rust-quality-reviewer)

- Event: gate result: fail
- Candidate: `33043491c534bf2d280e2e759a116f13c95e299e`
- Checks run: `git rev-parse HEAD` / `git status --short --branch` / ancestry checks from `0e5cbcdaee31b398a460ff7bd6878022dc658dce` through the candidate and from the candidate to `HEAD` → exit 0; HEAD is exactly the candidate, ancestry is valid, and the worktree is clean; complete commit-by-commit path audit and full diff from `0e5cbcd...` to `33043491...` → intermediate commits only append under `work-items/LINT-001.md`'s Log, while the candidate adds the final append-only log entry and changes only ADR-0001 through ADR-0006's header statuses and the six matching `docs/adr/README.md` index cells from `Proposed` to `Accepted`; no implementation, manifests, lockfile, or architecture prose changed; `git diff --check 0e5cbcd..33043491` → exit 0, no output; commit-message byte inspection showed literal `\n\n` characters before `Kanban: LINT-001`, and `git interpret-trailers --parse` → exit 0 with no output.
- Checks recommended but not run: `CARGO_BUILD_JOBS=4 CI=true just check`, because renewed Validation owns the exact-candidate gate; AC8 under a genuine non-root account, because Validation owns that environment-specific evidence and it remains open.
- Findings: F1 [blocking] `docs/git-workflow.md:38` — candidate `33043491c534bf2d280e2e759a116f13c95e299e` does not end with the required parseable `Kanban: LINT-001` trailer; its message contains literal `\n\nKanban: LINT-001`, so Git recognizes no trailer. This breaks the repository's required card-to-commit traceability.
- Scope audit: pass — the maintainer-approved ADR status changes and append-only LINT-001 logs are the only file changes after prior validated candidate `0e5cbcdaee31b398a460ff7bd6878022dc658dce`; implementation and architecture content are byte-for-byte unchanged.
- Next: rework the unpublished ADR-acceptance commit message so it ends with a real blank line followed by `Kanban: LINT-001`, producing a new candidate ID; then repeat Review and Validation. AC8 remains open for genuine non-root validation.

### 2026-09-14: LINT-001 malformed traceability trailers repaired in unpublished history (rust-implementer)

- Event: candidate change
- Candidate: `edc499b901ddf732bab8d1cec34e6f9acf888614`
- Checks run: created and verified recovery ref `refs/backup/lint-001-pre-trailer-rewrite-4fcd1ea` at old head `4fcd1eade37398924adde7ec4cb94dfef3224a64`; pairwise `git diff --quiet` gave exit 0 for old/new commits `1a29bfb8d318f94f6eeb50241eff6554c486c479`/`e978ad3797bef7e64361d7add2c8d0347e61ab3b`, `33043491c534bf2d280e2e759a116f13c95e299e`/`edc499b901ddf732bab8d1cec34e6f9acf888614`, and `4fcd1eade37398924adde7ec4cb94dfef3224a64`/`04a531dbe4b973e3d93357a2b5fa21ce88b6206c`; `git interpret-trailers --parse` on every commit in `f3e882c85b751a7a0b1cd276cc61190376055a0c..HEAD` gave `Kanban: LINT-001`; old/new review-evidence message hashes matched; work-item checksum remained `43fa7f357eba8f8470d45c95a9f416f6c36acf722778ae19df153419b90a3a6a`; final status was clean
- Checks recommended but not run: `CARGO_BUILD_JOBS=4 just check`, because this rework changed only unpublished commit metadata and every reconstructed tree is byte-identical to its previously checked counterpart
- Evidence: F1 fixed by replacing literal `\n\nKanban: LINT-001` text with a real blank line and parseable trailer in the rewritten release-readiness evidence commit and superseding candidate; old candidate `33043491c534bf2d280e2e759a116f13c95e299e` maps to new candidate `edc499b901ddf732bab8d1cec34e6f9acf888614`; later review-evidence commit recreated as `04a531dbe4b973e3d93357a2b5fa21ce88b6206c` with unchanged tree and message
- Next: rust-quality-reviewer, re-review candidate `edc499b901ddf732bab8d1cec34e6f9acf888614` for F1 closure

### 2026-09-14: LINT-001 rewritten superseding candidate re-reviewed — pass, F1 closed (rust-quality-reviewer)

- Event: gate result: pass
- Candidate: `edc499b901ddf732bab8d1cec34e6f9acf888614`
- Checks run: candidate existence and ancestry checks → exit 0; candidate is an ancestor of clean branch head `af1d3067276bd3456f1bc9ea0f386eab96cf34f4`, and recovery ref `refs/backup/lint-001-pre-trailer-rewrite-4fcd1ea` resolves to old head `4fcd1eade37398924adde7ec4cb94dfef3224a64`; tree-ID inspection and pairwise `git diff --exit-code` for old/new commits `1a29bfb8d318f94f6eeb50241eff6554c486c479`/`e978ad3797bef7e64361d7add2c8d0347e61ab3b`, `33043491c534bf2d280e2e759a116f13c95e299e`/`edc499b901ddf732bab8d1cec34e6f9acf888614`, and `4fcd1eade37398924adde7ec4cb94dfef3224a64`/`04a531dbe4b973e3d93357a2b5fa21ce88b6206c` → exit 0 for every pair, proving byte-identical trees; Git trailer parsing over every commit after `f3e882c85b751a7a0b1cd276cc61190376055a0c` through `HEAD` returned `Kanban: LINT-001`, and explicit `git interpret-trailers --parse` on the candidate returned the same trailer; commit-by-commit and complete diff audit from candidate to `HEAD` showed only 19 appended lines under `work-items/LINT-001.md`'s Log across two evidence commits; full ADR diff from prior validated candidate `0e5cbcdaee31b398a460ff7bd6878022dc658dce` showed exactly ADR-0001 through ADR-0006's header statuses and their six matching index cells changing from `Proposed` to `Accepted`; product-path and `git diff --check` audits gave exit 0, with no implementation, manifest, lockfile, architecture, brief, workflow-policy, or Git-policy drift; final worktree status was clean.
- Checks recommended but not run: `CARGO_BUILD_JOBS=4 CI=true just check`, because renewed Validation owns the exact-candidate gate; AC8 under a genuine non-root account, because Validation owns that environment-specific evidence and it remains open.
- Findings: none. Previous F1 is closed: candidate `edc499b901ddf732bab8d1cec34e6f9acf888614` ends with a real, parseable `Kanban: LINT-001` trailer, and the history rewrite changed commit metadata only.
- Scope audit: pass — rewritten commits are tree-identical to their recovery-ref counterparts; the accepted ADR status changes and append-only LINT-001 evidence logs remain the only content changes after `0e5cbcdaee31b398a460ff7bd6878022dc658dce`; no code or architecture content changed.
- Next: rust-validation-engineer, validate exact candidate `edc499b901ddf732bab8d1cec34e6f9acf888614`; retain AC8 as open until exercised under a genuine non-root account.

### 2026-09-14: LINT-001 superseding candidate validated — pass, all eleven ACs verified including genuine non-root AC8 (rust-validation-engineer)

- Event: gate result: pass
- Candidate: `edc499b901ddf732bab8d1cec34e6f9acf888614`
- Checks run: candidate pinning and ancestry/content audits gave exit 0 — detached HEAD exactly matched the candidate, its worktree was clean, it is an ancestor of card-branch head `779fb506a872f8adbdcdd27a4b56ff166da7b59e`, and every later commit changes only `work-items/LINT-001.md` with an append beneath `## Log`; implementation and command-contract paths are byte-identical to prior validated candidate `0e5cbcdaee31b398a460ff7bd6878022dc658dce`; ADR-0001 through ADR-0006 and their index rows all say `Accepted`, and the candidate has a parseable `Kanban: LINT-001` trailer. The initial bare `CARGO_BUILD_JOBS=4 just check` exited 127 before the gate began because the sandbox default PATH omitted the installed `just`; after restoring `/usr/local/cargo/bin`, `CARGO_BUILD_JOBS=4 just check` gave exit 0 (rustfmt clean, Clippy `-D warnings` clean, 69 tests passed and 0 failed, 0 doctest failures, rustdoc clean, cargo-deny “advisories ok, bans ok, licenses ok, sources ok”). `CARGO_BUILD_JOBS=4 CI=true just check` with the same installed tool directory on PATH gave exit 0 with locked Cargo operations and the same successful test/docs/dependency results. Each of the eleven AC integration binaries run independently via `CARGO_BUILD_JOBS=4 cargo test -p lint --test <name>` gave exit 0, totaling 20 passed and 0 failed. `CARGO_BUILD_JOBS=4 cargo build -p lint` gave exit 0. A direct AC8 invocation through `/usr/bin/setpriv --reuid=65534 --regid=65534 --clear-groups` observed euid 65534, egid 65534, and groups 65534 against a root-owned mode-000 Markdown file under a root-owned mode-755 searchable temporary parent; the exact-candidate binary produced zero stdout bytes, stderr naming the path with `cannot be read (Permission denied (os error 13))`, no panic text or file-content leakage, and exact exit 2; exact tempdir cleanup passed. `target/debug/lint docs` gave exit 0 with empty stdout/stderr. Removal of only `/workspace/lint/.worktrees/validate-LINT-001` succeeded and the disposable worktree is absent.
- Checks recommended but not run: formal performance benchmarking, because brief §13 provides no pass/fail budget; property/fuzz testing, because the architecture contract makes it discretionary and adding its dependency is outside Validation; timeout/shutdown/restart testing, because this short-lived local CLI has no service lifecycle or external dependency.
- Evidence: AC1 `cli_walk_single_file` — 1 passed; AC2 `cli_md009` — 3 passed; AC3 `cli_md047` — 4 passed; AC4 `cli_walk_directory` — 2 passed; AC5 `cli_multiple_paths_mixed` — 3 passed; AC6 `cli_no_path_given` — 1 passed; AC7 `cli_nonexistent_path` — 2 passed; AC8 `cli_unreadable_file` — 1 passed plus the non-vacuous uid/gid 65534 permission-denied probe above; AC9 `cli_walk_no_markdown_files` — 1 passed; AC10 `cli_md009_and_md047_same_file` — 1 passed; AC11 `cli_symlink_loop_does_not_hang` — 1 passed in 0.02s, with the full gate also covering hostile Markdown, oversized input, invalid UTF-8, and no-panic behavior. Verdict: pass; every in-scope criterion is verified on exact candidate `edc499b901ddf732bab8d1cec34e6f9acf888614`.
- Next: rust-release-steward, reassess readiness with the Accepted-ADR and genuine non-root AC8 lines closed; after a readiness pass, orchestrator may perform the maintainer-authorized local squash integration without pushing.

### 2026-09-14: LINT-001 ready for local squash integration; not released (rust-release-steward)

- Event: handoff
- Candidate: `edc499b901ddf732bab8d1cec34e6f9acf888614`
- Checks run: checkpoint status gave exit 0 and matched this reassessment at clean branch head `314d8ee914176d84210de855b76c8e213aeed5ae`; candidate existence and ancestry checks gave exit 0; product-content comparison from the candidate to HEAD was empty, with all four later commits changing only `work-items/LINT-001.md`; `main` remained `0c8864e4f272cd3dca7663abe95ab43f5287e3ae`; ADR-0001 through ADR-0006 and their index rows all said `Accepted`; `git remote -v` returned no remotes.
- Checks recommended but not run: `CARGO_BUILD_JOBS=4 CI=true just check` on the future squash-integration commit, because the orchestrator owns local integration; clean-checkout `CARGO_BUILD_JOBS=4 CI=true just check` and `CARGO_BUILD_JOBS=4 cargo run --release -p lint -- docs` against GitHub `main`, because no repository or delivery event exists yet.
- Evidence: the rewritten-candidate review entry records pass on exact candidate `edc499b901ddf732bab8d1cec34e6f9acf888614`; renewed validation records `CARGO_BUILD_JOBS=4 CI=true just check` exit 0, all eleven AC binaries passing, and a non-vacuous AC8 probe under uid/gid 65534 producing the expected permission-denied stderr and exit 2. The accepted ADRs close the stack-change approval line. Version remains `0.1.0`, appropriate for the first unpublished CLI; `CHANGELOG.md` already records LINT-001 under Unreleased. No data migration, configuration rollout, specialist review, signing, or provenance record applies.
- Release plan: the orchestrator may squash `feature/LINT-001-walking-skeleton` into local `main` under brief §10/§17’s existing delegation and the maintainer’s conditional authorization, then run `CARGO_BUILD_JOBS=4 CI=true just check` on the exact integration commit. No tag is planned because delivery is the GitHub `main` push rather than a versioned release. Future rollout is one maintainer-approved push. Rollback is `git revert <full-integration-commit-id>` followed by `git push origin main`; no migration cutoff exists. After the maintainer creates the repository, reviews the integrated commit, and gives push go, the maintainer runs `git remote add origin git@github.com:brunmmartins/lint.git` and `git push -u origin main`. A clean checkout of GitHub `main` must pass `CARGO_BUILD_JOBS=4 CI=true just check` and `CARGO_BUILD_JOBS=4 cargo run --release -p lint -- docs` with exit 0 and no findings before the maintainer declares delivery.
- Open readiness lines after local integration: exact-integration CI evidence, GitHub repository creation, maintainer pre-push review and push go/no-go, push-event delivery observation, clean-checkout post-release verification, and maintainer delivery declaration.
- Next: orchestrator, perform the authorized local squash integration, run the exact-integration gate, append its evidence, and stop before any remote or push action for maintainer review and go/no-go.

### 2026-09-14: LINT-001 squash-integrated into local `main`; exact-integration gate passed (orchestrator)

- Event: local integration; not delivery
- Candidate: `7cb0057d681ccc22d6a7e9af17c071fbf2196287`
- Checks run: clean `main` at `0c8864e4f272cd3dca7663abe95ab43f5287e3ae` shared the topic branch point; `git merge --squash feature/LINT-001-walking-skeleton` completed without conflicts; staged tree `c97911c27c8c341d7bd6d0566e9ffb3228832bc4` exactly matched topic-head tree `c97911c27c8c341d7bd6d0566e9ffb3228832bc4`; integration commit `7cb0057d681ccc22d6a7e9af17c071fbf2196287` had a parseable `Kanban: LINT-001` trailer; `CARGO_BUILD_JOBS=4 CI=true just check` on that exact clean commit gave exit 0 — `cargo fmt --all --check` clean, Clippy clean with `-D warnings`, 69 tests passed and 0 failed, doc tests passed, rustdoc completed with warnings denied, and cargo-deny reported advisories, bans, licenses, and sources all okay.
- Evidence: local `main` contains the reviewed and validated LINT-001 product tree as one squash integration commit. The full gate ran after the integration commit existed and before this append-only evidence record. No conflict resolution was required because `main` had not moved since the card branch point.
- Delivery status: not Released and not delivered — no Git remote exists, no GitHub repository or push event was observed, and the maintainer has not performed pre-push review or declared delivery.
- Next: maintainer, review local integration commit `7cb0057d681ccc22d6a7e9af17c071fbf2196287`, create the GitHub repository when ready, and give explicit push go/no-go; no remote, push, tag, or delivery declaration is performed by the orchestrator.
