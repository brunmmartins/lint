# Application brief: lint

Filled from the P template (`RUST_APPLICATION_PREREQUISITES.md`) on 2026-09-14. It records what is true
now, and is edited in place when an answer changes (P §1.2). Handbook citations are plain text: **K**
is the Kanban good-practices handbook, **S** the local development tech stack, **P** the prerequisites.
The decisions with consequences a maintainer would otherwise reopen are in [the ADR index](adr/README.md).

The minimum gate set (P §2.1) is answered: §3, §5, §6, §10, §18, and §19. The stack and dependency choices
inside §6 belong to the architecture stage and say so. Sections that are not needed yet say so, with the
stage that needs them (P §2.2).

## 3. Product definition and non-goals

- **Name:** `lint`, the repository and the binary name.
- **Outcome:** a person writing Markdown, or a CI job, runs one command over files or directories and
  gets every style problem reported with its location and rule, and can fix the safe ones automatically.
- **Problem today:** Markdown style problems (trailing whitespace, skipped heading levels, unlabelled code
  fences) are found by eye in review, or with a tool that needs a Node.js runtime. This is the
  requester's statement, not a measurement.
- **How a customer notices:** `lint docs/` prints `docs/guide.md:12:81 MD013/line-length ...` for each
  problem and exits non-zero; with no problems it prints nothing and exits 0.
- **In scope**, as chosen by the maintainer on 2026-09-14:
  - Lint the Markdown files named on the command line, and the Markdown files found by walking the
    directories named on it.
  - A core subset of twelve rules, identified by their markdownlint IDs and aliases: MD001
    `heading-increment`, MD009 `no-trailing-spaces`, MD010 `no-hard-tabs`, MD012 `no-multiple-blanks`,
    MD013 `line-length`, MD018 `no-missing-space-atx`, MD022 `blanks-around-headings`, MD025
    `single-h1`, MD031 `blanks-around-fences`, MD032 `blanks-around-lists`, MD040
    `fenced-code-language`, MD047 `single-trailing-newline`. Each rule's documented markdownlint
    behavior is the reference for what it reports.
  - Human-readable output, one finding per line as `path:line:col RULE message`.
  - A configuration file that enables or disables rules and sets rule options, such as the MD013 limit.
  - Inline disable and enable comments inside Markdown files.
  - JSON output for CI and editor integration.
  - Automatic fixing of findings whose fix is safe.
- **Non-goals:** the other markdownlint rules; byte-for-byte parity with markdownlint's output,
  configuration format, or edge-case behavior; custom or plugin rules; a language server or editor
  plugin; a watch mode; linting Markdown embedded in other file types; network access; packaging,
  installers, and publication to a registry.
- **Why now:** the request was made on 2026-09-14. There is no measured cost of delay.
- **Sunset condition:** the maintainer adopts another free linter that covers the same outcome without
  an extra runtime.

## 4. Customers, actors, and trust boundaries

| Actor | Human or machine | Internal or external | Trusted? | Authenticated how | Authorized by what |
|---|---|---|---|---|---|
| Person at a local shell | Human | Internal | Trusted user; the files and configuration they point at are untrusted content | None | The process has the user's own file permissions |
| CI job | Machine | Internal | Trusted runner; repository content is untrusted | None | The runner's file permissions |

- **Tenancy:** single user, one process per invocation.
- **Abuse cases:** hostile Markdown (very long lines, deeply nested lists or block quotes, huge files,
  invalid UTF-8), hostile configuration files, and directory trees with symbolic-link loops. A panic,
  a hang, or unbounded memory on any input is a failed acceptance criterion.
- **Writes:** only `--fix` writes, and only to the Markdown files being linted.
- **Expected volume:** a repository's worth of Markdown, from a handful of files to a few thousand.
- **Reading another actor's data:** not applicable; nothing is stored or sent.

## 5. The first delivering slice

- **Card:** LINT-001, the walking skeleton.
- **Outcome:** `lint <path>...` walks files and directories, parses Markdown, runs MD009 and MD047, prints
  findings as `path:line:col RULE message`, and exits with a status that tells a CI job whether problems
  were found. It crosses the CLI adapter, the file walker, the Markdown parser, the rule engine, and the
  text reporter, and it sets the extension points that LINT-002 to LINT-008 build on.
- **Acceptance criteria:** owned by `work-items/LINT-001.md`.
- **Excluded from the slice:** every rule other than MD009 and MD047, the configuration file, inline
  comments, JSON output, and `--fix`. Each is its own card (§17).
- **Size against the SLE:** there is no SLE yet (§17). Each card is judged small enough for one
  implementation pass.

## 6. Interface shape and stack profile

- **Profile:** `cli`. The interface is a command-line program; nothing is served, stored, or scheduled.
- **Crates and third-party dependencies:** the maintainer allowed free-to-use crates on 2026-09-14, for
  example a CommonMark parser and an argument parser. Which crates, and the crate layout, are chosen by
  `rust-solution-architect` for LINT-001 and recorded as ADRs.
- **Profiles not chosen, and their triggers:** `library` when another crate consumes the rule engine;
  `http`, `persistence`, `worker`, and `full-local` only if a non-goal is reversed.
- **Obligations accepted** (P §6, S §4.2):
  - Exit-code contract, proposed on 2026-09-14 and confirmed by LINT-001's acceptance criteria: 0 when no
    finding remains, 1 when at least one finding remains, 2 on a usage, configuration, or I/O error.
  - stdout carries findings (text or JSON); stderr carries errors, warnings, and usage.
  - Argument stability: rule IDs, aliases, flags, the configuration keys, and the JSON shape are the CLI
    contract. Changing one is a breaking change (§7).

## 7. Consumers, contracts, and compatibility

- **Consumers:** the maintainer at a local shell, and CI jobs that read the exit status or the JSON.
- **Interface kinds emitted:** the CLI (§6), the configuration file format, the inline comment syntax,
  and the JSON output shape.
- **Contract direction:** hand-maintained, checked by integration tests against the built binary.
- **Versioning:** `0.1.0`, unpublished, with no semver promise yet.
- **MSRV and default features:** no MSRV promise and no features.
- **Deprecation window and who must be told:** not yet; needed before the first external user.

## 8. Data: state, classification, and retention

- **Durable state:** none owned by `lint`. `--fix` rewrites the user's own Markdown files in place.
- **Classification:** the Markdown files may hold anything the user wrote. `lint` echoes only locations,
  rule IDs, and messages, never file contents, into its output.

## 9. External dependencies and integrations

- **Runtime:** the local file system only. No network, no environment variables beyond what the
  architecture stage decides for configuration discovery, and no subprocesses.
- **Crates:** chosen at the architecture stage, each free to use and allowed by `deny.toml`.
- **Free to use (P §1.4):** every tool in use is free: `rustup`, Cargo, `rustfmt`, Clippy, `just`, and
  `cargo-deny`. The hosted service is GitHub, on the Free plan with a public repository, once the
  maintainer creates it. The limit that would force a paid plan: a private repository loses branch
  protection and caps GitHub Actions minutes. The free fallback is `CI=true just check` run locally.

## 10. Delivery point and commitment point

- **Commitment point:** Options → Ready. For LINT-001 to LINT-008 this is the maintainer's request of
  2026-09-14, which named the rule set and the four features.
- **Integration:** with the maintainer's instruction of 2026-09-14, a card whose review and validation
  pass is squash-merged into the local `main` without a further question. That is integration, not
  delivery.
- **Delivery point:** the change is on `main` at github.com/brunmmartins/lint. Merge and availability
  coincide because a consumer builds `lint` from the repository. GitHub's push event is the delivery
  timestamp. The repository does not exist yet; the maintainer creates it and decides every push.
- **Post-release verification:** the commit on GitHub's `main` is the one verified, and from a clean
  checkout of it `CI=true just check` passes and `lint` run over the repository's own `docs/` exits as
  its acceptance tests predict.
- **Who may declare an item delivered:** the maintainer.
- **Release cadence and approvals:** none; there are no versioned releases.

## 11. Target environment and operation

- **Runs on:** any platform Rust 1.98.0 supports, as a native process. Built and checked on Linux
  aarch64.
- **Environments:** developer machines and CI runners. No deployment, operator, or on-call.
- **Rollback:** `git revert` of the commit on `main`. Files changed by `--fix` are the user's to revert
  with their own version control.
- **Feature flags:** none.

## 12. Configuration and secrets

- **Configuration:** a project configuration file for rules and rule options (§3). Its name, format,
  discovery rules, and precedence against flags are decided for LINT-005. No secrets.

## 13. Service levels and performance budget

- Interactive and CI use. Assumption, not a measurement: a repository of a few thousand Markdown files
  lints in seconds on a developer machine. Memory is bounded by the largest single file plus the
  findings. There is no measured user.

## 14. Observability and operational readiness

- None beyond the output. Diagnostics are the findings on stdout, errors on stderr, and the exit status.

## 15. Security, privacy, and compliance

- **Assets:** the correctness of findings, and the integrity of files that `--fix` rewrites.
- **Untrusted input:** file contents, directory trees, configuration files, and inline comments. No
  input may cause a panic, a hang, or unbounded recursion. `--fix` never writes a file it did not read,
  never writes outside the paths being linted, and leaves a file untouched when it cannot apply every
  selected fix safely.
- **Personal data, regime, audit logging:** none.
- **Vulnerability reporting:** GitHub private vulnerability reporting once the maintainer creates the
  repository and enables it; until then, the maintainer privately through GitHub (see
  [SECURITY.md](../SECURITY.md)).
- **Dependency policy exceptions:** none.
- **Specialist review before release:** none required.

## 16. Service definition and sources of demand

- **Owner:** the maintainer, for the `lint` repository.
- **How requests arrive:** GitHub issues on brunmmartins/lint, once it exists.
- **Accepts:** changes that stay inside the §3 outcome. Anything in the non-goals is declined, or
  becomes a change to this brief first.
- **Support hours:** none promised.

## 17. Flow policy and decision rights

- **Board:** `unknown: maintainer, review by 2026-09-28`. Until a board exists, the latest gate result in
  each work-item log stands in for card state, and the orchestrator's session report says so.
- **Work-item IDs:** `LINT-NNN`, assigned in order by the orchestrator under the maintainer's request.
- **Cards committed on 2026-09-14:**

  | Card | Slice | Depends on |
  |---|---|---|
  | LINT-001 | Walking skeleton: walk paths, parse, engine, MD009, MD047, text output, exit codes | none |
  | LINT-002 | Whitespace and length rules: MD010, MD012, MD013 | LINT-001 |
  | LINT-003 | Heading rules: MD001, MD018, MD022, MD025 | LINT-001 |
  | LINT-004 | Block rules: MD031, MD032, MD040 | LINT-001 |
  | LINT-005 | Configuration file: enable, disable, rule options | LINT-001 |
  | LINT-006 | Inline disable and enable comments | LINT-001 |
  | LINT-007 | JSON output | LINT-001 |
  | LINT-008 | `--fix` for safe fixes | LINT-001 |

- **States:** Options → Ready → Development → Review → Validation → Released.
- **WIP:** one card per working tree. After LINT-001 is integrated, LINT-002 to LINT-008 run in parallel,
  each in its own Git worktree.
- **Classes of service, SLE percentile, day counting:** not yet; needed by the first forecast.

| Decision | Decided by | Consulted | Escalation |
|---|---|---|---|
| Product scope and priority | Maintainer | None | None |
| Architecture and technology, including ADR acceptance | Maintainer; development may proceed on Proposed ADRs (instruction of 2026-09-14) | None | None |
| Security exception | Maintainer | None | None |
| Local integration into `main` | Delegated to the orchestrator once review and validation pass (instruction of 2026-09-14) | None | Maintainer |
| Push, release go / no-go, declaring delivery | Maintainer | None | None |
| Rollback during an incident | Maintainer | None | None |

## 18. Toolchain, repository, and versioning policy

These choices carry over the policy the maintainer set for `calc` on 2026-09-14. That carry-over is an
assumption, cheap to change before the first push.

- **Repository:** `git@github.com:brunmmartins/lint.git`, public, on GitHub Free, to be created by the
  maintainer. No remote is configured yet. Commits use the maintainer's GitHub noreply address.
- **Licence:** `MIT OR Apache-2.0`, in `LICENSE-MIT` and `LICENSE-APACHE`. Third-party licences are
  governed by `deny.toml`, which allows only licences that permit free use.
- **Copyright holder:** "The lint contributors". Assumption: the maintainer has not named a holder.
- **Default branch:** `main`. Branch protection is not configured; the maintainer enables it on GitHub.
- **Merge strategy:** squash merge, so one card becomes one commit on `main`. Topic branches are rebased
  onto `main` while unpublished. Published history on `main` is never rewritten.
- **Signing:** not required. There is no provenance requirement and no verification path (S §7.21).
- **Toolchain:** pinned to `1.98.0`, the current stable release and the only toolchain on the build
  machine, with `clippy` and `rustfmt`. Update it deliberately, as a card.
- **Edition:** 2024. No MSRV promise.
- **`Cargo.lock`:** committed, because the workspace produces a binary (S §19.2).
- **Release profile:** default; not yet decided otherwise, needed before a first release build.
- **Changelog:** `CHANGELOG.md`, with entries under Unreleased, written with each card.
- **Review:** agent review and validation are the gate for local integration (§10, §17); the
  maintainer's review is required before a push. No CODEOWNERS.

## 19. Local environment capability check

Observed on 2026-09-14 on the build machine.

| Capability | Required by | Observed | Gap and decision |
|---|---|---|---|
| Rust toolchain and Cargo | every profile | rustc and cargo 1.98.0, rustup 1.29.0, aarch64 | None |
| Registry access | third-party crates | crates.io index reachable (HTTP 200) | None |
| `just` | the command contract | 1.58.0 | None |
| `cargo-deny` | dependency policy | 0.20.2 | None |
| `cargo-nextest` | optional | Not installed | `just test` falls back to `cargo test` (S §2.4) |
| `cargo-audit` | optional | Not installed | `cargo-deny` covers advisories |
| Container runtime, database | persistence, full-local | Not installed | Not needed by the `cli` profile |
| Build capacity for parallel cards | parallel worktrees | 14 CPUs, 7 GiB RAM | Parallel builds cap Cargo jobs per worktree |
| CI runner | merge gates | None configured | `CI=true just check` run locally is the gate until a workflow is added |
| Free-to-use terms | P §1.4 | All tools above are free | None |

## 20. Open choices

| Open choice | Resolution |
|---|---|
| Pinned toolchain vs moving `stable` | Pinned `1.98.0` (§18) |
| MSRV promise | None (§18) |
| `Cargo.lock` policy | Committed (§18) |
| Topic-branch update | Rebase while unpublished (§18) |
| Merge strategy | Squash (§18) |
| Signing | Not required (§18) |
| Crates and crate layout | Not yet; needed by `rust-solution-architect` for LINT-001 |
| Configuration file name, format, and discovery | Not yet; needed by `rust-solution-architect` for LINT-005 |
| Inline comment syntax | Not yet; needed by `rust-intake-planner` for LINT-006 |
| JSON output shape | Not yet; needed by `rust-solution-architect` for LINT-007 |
| Which fixes count as safe | Not yet; needed by `rust-intake-planner` for LINT-008 |
| Board location | `unknown: maintainer, review by 2026-09-28` (§17) |
| Calendar vs working days; SLE percentile | Not yet; needed by the first forecast (§17) |
| Release-profile settings | Not yet; needed before a first release build (§18) |
