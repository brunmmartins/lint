# ADR-0005: Walk directories without following symlinks, using no third-party walker crate

- Status: Accepted
- Date: 2026-09-14
- Owners: maintainer (brief §17, "Architecture and technology")
- Work item: `work-items/LINT-001.md`
- Supersedes: none

## Context

AC11 requires that `lint` never panics or hangs on any input in the acceptance-criteria set, "including
on a directory tree containing a symbolic-link loop." Brief §4 names symlink loops explicitly as an abuse
case. AC9 requires an empty walk (directory with no Markdown files) to be silent and exit 0 — not an
error. AC4/AC5 require recursive directory walking with a "stable and documented" order.

A symlink loop (for example `a/link -> ..` or `a -> b`, `b -> a`) only causes non-termination if the
walker treats a symlink-to-directory as a directory to recurse into. If the walker never follows a
symlink as a directory, a cycle cannot form structurally — there is no traversal state to bound or detect,
because the traversal graph itself has no back-edges. This is a stronger guarantee than a depth counter or
a visited-inode set, which only bound the damage instead of ruling it out.

`walkdir` (the common third-party crate for this) defaults to not following symlinks (`follow_links(false)`),
which would give the same non-following behavior, but at the cost of a new dependency whose extra
features (symlink-following, min/max depth options, custom sort via `sort_by`, parallel walking) this
card does not need — S §20.5 Q1/Q5: none of that is part of `lint`'s business language, and the realistic
replacement cost of a hand-written iterative walker is small (one module, one call site, per ADR-0002).

## Decision

`apps/lint`'s `StdWalker` adapter walks directories using `std::fs::read_dir`, with an explicit stack
(`Vec<PathBuf>`) rather than recursive function calls, so traversal depth cannot cause native stack
overflow regardless of how deep a (non-cyclic) real directory tree goes. Before recursing into any entry,
the walker calls `std::fs::symlink_metadata` (which does not follow the final symlink component) and
recurses only when the entry is a directory and is not a symlink. A symlink is otherwise treated as a
leaf: if it is a regular file, it may still be linted as a Markdown file by extension (§ below); if it
points to a directory, it is never traversed. No third-party walker crate is added.

Markdown-file selection for directory-discovered entries: extension is `.md` or `.markdown`,
case-insensitive. A file path given directly on the command line is always linted regardless of
extension, since giving it by name is explicit user intent (AC1–AC3, AC7, AC8, AC10 pass bare file paths
with no stated extension requirement).

Traversal order (for AC5's "stable and documented" order): within a directory, entries are sorted by file
name before being pushed onto the stack, and CLI-argument inputs are processed in the order given on the
command line. This makes the walk deterministic across platforms and directory-entry orders returned by
the OS.

## Options considered

| Option | For | Against | Evidence |
|---|---|---|---|
| Hand-written iterative walker, never follows symlinks (chosen) | No new dependency; symlink loops are structurally impossible, not just bounded; deterministic sorted order is easy to add at the same site | Reimplements a small amount of logic `walkdir` already has | S §20.5; brief §4 abuse case |
| `walkdir` crate, `follow_links(false)` (its default) | Well-tested, handles more edge cases (e.g. certain Windows reparse points) than a hand-written walker might | New dependency whose extra surface (symlink-following, depth limits, parallelism) is unused; still needs a custom `sort_by` for AC5's determinism, so it does not remove all hand-written code anyway | crates.io, checked 2026-09-14: MIT licensed, would clear `deny.toml` if chosen later |
| Follow symlinks, detect cycles with a visited-inode/device set | Would let a user opt into linting symlinked content | Adds complexity (platform-specific inode handling, unclear behavior on Windows) to solve a case brief §3/§5 does not ask for; a visited-set only bounds a cycle, it does not prevent one by construction, so it is a weaker guarantee for the same abuse case AC11 tests | Brief §4 abuse case list; AC11 |
| Recursive function calls (not iterative), never follow symlinks | Simpler code than an explicit stack | Recursion depth is bounded only by the native call stack; a very deep (but real, non-cyclic) directory tree could still overflow the stack, which is exactly the "unbounded recursion" risk the work item's Constraints section calls out | `work-items/LINT-001.md` Context → Constraints, "no input may cause a panic, a hang, or unbounded recursion" |

## Consequences

- Easier: AC11 (symlink loop) is satisfied by construction, not by a runtime guard that could be
  disabled or miscounted; no new dependency to vet for this card.
- Harder: if a future card wants `lint` to optionally follow symlinks (not currently in any brief
  non-goal reversal), that is new adapter logic and a new ADR, not a flag on an existing dependency.
- Newly constrained: `StdWalker` must use `symlink_metadata`, never plain `metadata`, before deciding
  whether to recurse — a reviewer can check this at the one call site.
- Left unverified: behavior on Windows reparse points / junctions is not tested by this card (brief §11:
  target build/check platform is Linux aarch64); flagged under "Left unverified" in the contract.

## Follow-up

Revisit if a later card needs symlink-following (would need a new ADR and an explicit opt-in flag, never
a silent default change, since AC11's guarantee must not regress), or if `apps/lint`'s walker grows
enough ad hoc logic that adopting `walkdir` becomes cheaper than maintaining it by hand.
