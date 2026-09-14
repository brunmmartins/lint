# ADR-0006: I/O faults take exit-code precedence over findings; output order is input order then sorted walk order

- Status: Accepted
- Date: 2026-09-14
- Owners: maintainer (brief §17, "Architecture and technology")
- Work item: `work-items/LINT-001.md`
- Supersedes: none

## Context

Brief §6 fixes the exit-code contract's three values (0 no finding, 1 finding present, 2 usage/
configuration/I/O error) but not their precedence when more than one condition is true in the same run.
AC5 requires multiple paths, mixing a clean file, a dirty file, and a directory, to report findings from
every path and exit 1 if any produced a finding — but AC5 does not mix in a nonexistent or unreadable
path, so the brief and the work item leave one question genuinely open: what is the exit code when one
path produces findings (would be exit 1) and a different path in the same invocation produces an I/O
fault (would be exit 2, per AC7/AC8, each tested only as a single-path case)? AC5 also requires findings
to print in a "stable and documented" order, which needs a concrete rule.

Brief §7 states the CLI contract, once set, is a compatibility surface — this precedence and ordering
rule becomes part of that surface the moment `apps/lint` ships, so it is fixed here rather than left to
the implementer's discretion.

## Decision

**Exit-code precedence:** if any input produced an I/O-class fault (not found, unreadable, over the size
bound, invalid UTF-8), the process exits 2, regardless of whether other inputs also produced findings.
Only if no input produced a fault does the process exit 1 (any finding present) or 0 (none). Rationale:
class 2 covers "the invocation itself could not be trusted to have inspected everything asked of it" —
mixing that signal into class 1 (a normal "problems found" result a CI job might otherwise auto-triage
differently from a broken invocation) would hide the more serious condition. All findings and all fault
messages are still printed in the same run — the exit code does not suppress output for the paths that
did succeed.

**Output order:** paths are processed in the order given on the command line (left to right); for a
directory input, contained Markdown files are visited in the sorted-by-file-name order ADR-0005 fixes;
within one file, findings are sorted by `(line, column)` before printing. Findings print to stdout in
that order; fault messages print to stderr in the same input order (stdout and stderr are separate
streams, so their relative interleaving is not part of this contract — only each stream's own order is).

## Options considered

| Option | For | Against | Evidence |
|---|---|---|---|
| Fault (2) takes precedence over findings (1) (chosen) | Distinguishes "ran clean and found problems" from "could not fully run"; matches class 2's description in brief §6 as covering "usage, configuration, or I/O error" as the more severe category; a CI job branching on exit code should be told the harsher truth | A path that failed to read produces no findings for that path, which could read as "clean" if a caller only checks for a nonzero exit without reading stderr — mitigated by requiring the fault to still be printed to stderr | Brief §6; work item AC7/AC8 |
| Findings (1) take precedence over faults (2) | Simpler mental model ("any problem, findings or faults, is exit 1") | Collapses two different failure classes brief §6 deliberately separates; a CI job could not distinguish "please fix your Markdown" from "please fix your invocation" by exit code alone | Brief §6's three-way exit code split is evidence the two classes are meant to stay distinguishable |
| Sum/bitmask exit codes (e.g. 3 = both) | Encodes both conditions in one code | Brief §6 fixes exactly three exit values (0/1/2); inventing a fourth is a new, uncommitted contract element the brief does not ask for | Brief §6 |
| Defer (leave precedence to the implementer) | N/A | The CLI/exit-code contract is a compatibility surface (brief §7) and an escalation trigger; leaving it to Development risks an undocumented, hard-to-reverse choice made without ADR review | Brief §7; `docs/adr/README.md` |

## Consequences

- Easier: `tests/cli_multiple_paths_mixed.rs` (AC5) and any future test mixing a fault with a finding has
  an unambiguous expected exit code; `rust-implementer` does not have to invent this rule mid-Development.
- Harder: nothing identified — this is additive precision on top of brief §6, not a new obligation.
- Newly constrained: adding a fourth exit-code class later, or changing this precedence, is a breaking
  change to the CLI contract (brief §7) and needs its own ADR that supersedes this one.
- Left unverified: no acceptance criterion in this card actually exercises the mixed fault-plus-finding
  case end-to-end (AC5 mixes clean/dirty/directory, not a nonexistent path, with a finding-producing
  path). `rust-implementer` should add this as an additional test beyond the eleven ACs' minimum, and if
  not, `rust-validation-engineer` verifies this ADR's precedence rule is at least unit-tested at the
  application layer (`decide_exit_code`, per the contract's Test plan).

## Follow-up

Revisit if the maintainer wants CI tooling to distinguish "lint found problems" from "lint could not run"
in a way exit codes 0/1/2 cannot express (for example, brief §7's future JSON output, LINT-007, could
carry this distinction more richly than the exit code alone).
