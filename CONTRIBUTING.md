# Contributing to lint

## Before you start

Install Git, Rust through `rustup`, and Cargo. [`just`](https://github.com/casey/just) is optional but
recommended; every recipe is also documented below.

```bash
just bootstrap
```

Create a short-lived branch from `main` and keep each change focused on one outcome.

## Commands

The `justfile` is the command contract. Keep this table synchronized with it whenever a recipe changes.

| Recipe | Runs | When |
|---|---|---|
| `just bootstrap` | Checks for `git`, `rustup`, and `cargo`; installs the toolchain from `rust-toolchain.toml`; reports whether `cargo-nextest` and `cargo-deny` are available | Once per clone and after a toolchain change |
| `just fmt` | `cargo fmt --all` | Before committing |
| `just lint` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | While editing |
| `just check-fast` | Formatting, compilation, and workspace library tests | While editing |
| `just test` | All workspace tests and documentation tests | Before committing |
| `just docs` | Builds workspace API documentation with warnings denied | When public API documentation changes |
| `just deny` | Runs dependency advisory, license, ban, and source checks | When dependencies change |
| `just check` | Formatting, linting, tests, documentation, and dependency checks | Before requesting review |

Running `just` with no arguments lists the recipes. When `CI` is set, Cargo commands use `--locked` and
a missing `cargo-deny` is an error rather than a visible local skip.

If `just` is unavailable, run the underlying commands directly:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features --all-targets --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo deny check
```

## Branches, commits, and review

- Branch from `main` using a short descriptive name such as `feature/add-rule` or `fix/path-order`.
- Write commit subjects in the imperative and explain the reason for non-obvious changes in the body.
- Stage deliberately and inspect `git diff --staged` before committing.
- Include tests for behavior changes and run `just check` before requesting review.
- Call out public API, dependency, security, compatibility, and migration effects in the review request.

## Describing a change

```markdown
## Problem and outcome
What was wrong or missing, and what changes for users.

## Approach
The implementation and any alternatives that were rejected.

## Validation
Commands run and their observed results.

## Risk
Failure modes, compatibility concerns, and rollback or forward-fix options.
```
