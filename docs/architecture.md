# Architecture

This describes how lint is put together **now**. The reasons for its shape are ADRs, indexed
in [adr/README.md](adr/README.md). If this document changes shape, an ADR changes with it.

## 1. Context

`lint` is a local command-line tool: a person at a shell, or a CI job, runs `lint <path>...` over
Markdown files and directories and gets style findings on stdout with a exit status that tells a human or
a CI job, without reading output, whether problems were found or the invocation itself failed (brief §3,
§4, §6). There is no server, no store, no queue, and no deployment — every run is a single local process
reading files with the invoking user's own file permissions (brief §4, §9, §11). LINT-001 is the walking
skeleton: it establishes the CLI/output/exit-code contract (brief §7) and the crate layout every later
card (LINT-002–LINT-008) builds on. See [ADR-0001](adr/ADR-0001-cli-stack-profile.md) and
[ADR-0002](adr/ADR-0002-crate-layout.md).

## 2. Stack profile

Profiles in use: `cli` only (brief §6), confirmed by
[ADR-0001](adr/ADR-0001-cli-stack-profile.md). That ADR also lists the profiles not chosen and what would
add each one (`library`, if another crate ever consumes the rule engine; `http`, `persistence`, `worker`,
`full-local` only if a non-goal in brief §3/§5 is reversed). Capabilities are additive, and none is
adopted speculatively (S §3.5, §4; P §6).

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

For `lint` concretely (LINT-001, [ADR-0002](adr/ADR-0002-crate-layout.md)): `apps/lint` (the only
frameworks/infrastructure layer) depends on `lint-application`, which depends on `lint-domain`, which
depends on neither. `lint-domain` never depends on `pulldown-cmark`, `clap`, `std::fs`, or `std::env`.

## 4. Crates

Defined by [ADR-0002](adr/ADR-0002-crate-layout.md) for LINT-001, and created in the LINT-001 candidate
commit: each directory, its `Cargo.toml`, and its path in the root `Cargo.toml` `members` list.

| Crate | Path | Depends on | Responsibility |
|---|---|---|---|
| `lint-domain` | `crates/domain` | (none) | Value objects (`Location`, `RuleId`, `Finding`), the `Document` type and its validating constructor, the `Rule` trait, and the MD009/MD047 rule implementations. No I/O, no framework types. |
| `lint-application` | `crates/application` | `lint-domain` | The `Walker`, `SourceReader`, and `MarkdownParser` ports; the `LintFault` taxonomy; the `run_lint` use case; the pure `format_finding` and `decide_exit_code` functions. |
| `lint` (binary) | `apps/lint` | `lint-application`, `lint-domain` | Composition root (`main.rs`) and every adapter: `cli.rs` (`clap`, [ADR-0004](adr/ADR-0004-argument-parser-crate.md)), `walker.rs` ([ADR-0005](adr/ADR-0005-walker-no-symlink-follow.md)), `reader.rs`, `parser.rs` (`pulldown-cmark`, [ADR-0003](adr/ADR-0003-commonmark-parser-crate.md)). |

Every crate in this table is listed in `members` in the root `Cargo.toml`, and every member is in this
table. Split a crate only at a real responsibility, dependency, compilation, or reuse boundary (K §10.2).
No adapter (walker, reader, parser, CLI) gets its own crate yet — nothing outside `apps/lint` consumes
them (ADR-0002).

## 5. Composition roots

One binary, `apps/lint`, and therefore one composition root: `apps/lint::main`. It is the only code that
knows every concrete technology (`pulldown-cmark`, `clap`, `std::fs`). Startup order for LINT-001: parse
arguments (`clap`; a missing/invalid argument exits via clap's own usage-error path, stderr + exit 2,
[ADR-0004](adr/ADR-0004-argument-parser-crate.md)); construct the concrete adapters (`StdWalker`,
`StdSourceReader`, `PulldownMarkdownParser`); call `lint_application::run_lint`; format and print
findings to stdout and faults to stderr in input order ([ADR-0006](adr/ADR-0006-exit-code-precedence-and-output-order.md));
compute the final exit code (fault present → 2, else finding present → 1, else 0, ADR-0006); return
`std::process::ExitCode` from `main` so stdout/stderr are flushed through the normal shutdown path
(K §24.3). There is nothing that "accepts traffic" to half-construct — a CLI invocation either completes
or exits; there is no partial-startup state to protect.

## 6. Ports, adapters, and replacement boundaries

Defined for LINT-001 by the architecture contract in `work-items/LINT-001.md` and
[ADR-0002](adr/ADR-0002-crate-layout.md)–[ADR-0005](adr/ADR-0005-walker-no-symlink-follow.md).

| Port (in `lint-application`, except `Rule`) | Adapter (in `apps/lint`, except `Rule` impls) | Replacement boundary |
|---|---|---|
| `Walker` | `StdWalker` (`std::fs`, iterative, never follows a symlink as a directory) | Only implementation; no runtime selection yet |
| `SourceReader` | `StdSourceReader` (`std::fs`, size-bounded before allocation) | Only implementation; no runtime selection yet |
| `MarkdownParser` | `PulldownMarkdownParser` (wraps `pulldown-cmark` 0.13.4) | Only implementation; replacing the parser touches only this one adapter module |
| `Rule` (in `lint-domain`) | `Md009`, `Md047` (in `lint-domain`, pure policy, no adapter needed) | `dyn Rule` in a static registry — the engine already iterates a heterogeneous, extensible list; LINT-002–LINT-004 add more implementations of the same port |

No port has two adapters yet, so no contract test suite exists yet (S §20.2); one becomes relevant if a
port ever gets a second implementation (for example a fake `Walker`/`SourceReader`/`MarkdownParser` used
in `lint-application`'s own tests, per the Test plan in `work-items/LINT-001.md`).

## 7. Configuration

Not yet. The configuration file arrives with LINT-005 (brief §12). No environment variables or secrets.
