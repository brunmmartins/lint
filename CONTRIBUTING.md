# Contributing to lint

## Before you start

1. Set up your machine with [docs/local-development.md](docs/local-development.md).
2. Start from a card the board shows as committed. Each branch is for one card
   ([docs/workflow-policy.md](docs/workflow-policy.md)).
3. Create or update that card's record in [`work-items/`](work-items/README.md). If the work settles a
   decision, record it as an ADR in [`docs/adr/`](docs/adr/README.md).

## Commands

The `justfile` is the command contract (S §16.1). This table lists what each recipe runs, so the
project stays usable without `just`. Every recipe is listed here, and every listed recipe exists.
When you change one, change the other in the same commit.

| Recipe | Runs | When |
|---|---|---|
| `just bootstrap` | Checks for `git`, `rustup`, and `cargo`; runs `rustup toolchain install --no-self-update` for `rust-toolchain.toml`; reports `cargo-nextest` and `cargo-deny`; runs `git config --local core.hooksPath .githooks` if `.githooks/` exists | Once per clone, and after a toolchain change |
| `just fmt` | `cargo fmt --all` | Before committing |
| `just lint` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | While editing |
| `just check-fast` | `cargo fmt --all --check`, then `cargo check --workspace --all-targets`, then `cargo test --workspace --lib` | While editing |
| `just test` | `cargo nextest run --workspace --all-features --no-tests=warn`, or `cargo test --workspace --all-features --all-targets` without nextest; then `cargo test --doc --workspace --all-features` | Before committing |
| `just docs` | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` | When public docs change; add `--open` to browse |
| `just deny` | `cargo deny check` | When dependencies change |
| `just check` | `cargo fmt --all --check`, then `just lint test docs deny` | Before requesting review |

Running `just` with no arguments lists the recipes.

**CI runs `just check` with `CI` set.** That adds `--locked` to every Cargo command, so a stale
`Cargo.lock` fails the build instead of being rewritten (K §20.4). It also turns a missing
`cargo-deny` from a visible skip into a failure. CI may run more than this, such as a feature matrix,
extra targets, or an MSRV job. It must not run these same checks differently (S §16.5). If CI and a
local run disagree, record that as a defect instead of rerunning.

## Branches, commits, and review

The full policy is in [docs/git-workflow.md](docs/git-workflow.md). In short:

- Use a short-lived branch from `main`, named `<prefix>/<card-id>-<slug>`.
- Write commit subjects in the imperative. The body says why. Add a `Kanban: <card-id>` trailer.
- Stage deliberately. Inspect `git diff --staged` rather than habitually running `git add .`.
- Integration uses squash merge, so one card becomes one revertible commit on `main`. The squash
  message keeps the card ID and the motivation (S §7.12). The required check is `CI=true just check` on the exact
  commit, run locally. Agent review and validation gate local integration into `main` (brief §10, §17);
  the maintainer's review is required before any push.

## Describing a change for review

Adapted from K §30.2. Leave out lines that do not apply, but do not leave out the risk section.

```markdown
## Problem and outcome
Card: <card-id>. What changes for the user or system.

## Approach
The design, and the alternatives that were rejected.

## Validation
- [ ] `just check` passes at <commit>
- [ ] Tests cover failure and edge behavior, not only the happy path
- [ ] Documentation changed with the behavior
- [ ] Dependency, security, compatibility, and migration effects reviewed

Commands and observed results:

## Risk and delivery
- Failure modes:
- Rollout or activation:
- Rollback or forward fix:
- Post-release verification:

## Review notes
Unsafe code, concurrency, public API changes, generated files, and anything that needs a specialist.
```

A change is done when it meets the Definition of Done in
[docs/workflow-policy.md](docs/workflow-policy.md), not when it merges.
