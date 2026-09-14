# ADR-0001: Adopt only the `cli` stack profile for `lint`

- Status: Accepted
- Date: 2026-09-14
- Owners: maintainer (brief §17, "Architecture and technology")
- Work item: `work-items/LINT-001.md`
- Supersedes: none

## Context

Brief §6 already states the interface is a command-line program: "nothing is served, stored, or
scheduled." Brief §9 confirms no network, no subprocesses, and no durable state. LINT-001 is the first
card and the first crate(s) in the repository (S §4), so this ADR is the first place a profile is
actually committed rather than merely stated.

S §4 lists five stack profiles (`cli`, `http`, `persistence`, `worker`, `full-local`), each adding
obligations that need answers before code. None of LINT-001's eleven acceptance criteria need a served
port, a store, a queue, or a Compose topology — they need a process that reads files named on argv,
walks directories, and writes to stdout/stderr with an exit code (AC1–AC11).

## Decision

`lint` adopts the `cli` profile only, for this card and until an acceptance criterion in a later card
forces another profile. Per S §4 this obliges:

- The exit-code contract (brief §6: 0 no finding, 1 finding present, 2 usage/configuration/I/O error).
- The stdout/stderr split (findings on stdout, everything else on stderr).
- Argument stability: once set, the CLI shape is a compatibility surface (brief §7).

Concretely: `domain` and `application` crates hold the profile-independent logic, and a thin `apps/lint`
binary crate is the only place argument parsing, file-system access, and process exit live (S §4's `cli`
row). See ADR-0002 for the crate split itself.

## Options considered

| Option | For | Against | Evidence |
|---|---|---|---|
| `cli` only (chosen) | Matches every AC; smallest profile; no obligations beyond exit code, stream split, argument stability | None found | Brief §6 states the interface kind directly; brief §9 confirms no network/persistence/scheduling |
| `cli` + `library` (expose the rule engine as a reusable crate now) | Would make future editor/LSP integration cheaper | Brief §6 says `library` is added "when another crate consumes the rule engine" — no such consumer exists yet; adding it now is speculative (K §1.1, S §3.5) | Brief §6, "Profiles not chosen, and their triggers" |
| `http`, `persistence`, `worker`, `full-local` | N/A | None of LINT-001's ACs need serving, storing, or scheduling; brief §6 explicitly excludes them unless a non-goal is reversed | Brief §6 |
| Defer (write no profile ADR yet) | N/A | `docs/architecture.md` §2 already commits this ADR's number and scope before any code exists; deferring leaves the profile undocumented while code is being designed | `docs/architecture.md` §2 |

## Consequences

- Easier: the crate layout stays small (three crates total, ADR-0002); no `.env.example`, no
  `compose.yaml`, no health/readiness contract, no shutdown-deadline design is needed for this card.
- Harder: nothing yet — no acceptance criterion is blocked by staying at `cli` only.
- Newly constrained: adding any other profile later (for example `library` once an editor integration
  wants the rule engine) is itself a new ADR with its own obligations (S §4), not an amendment to this
  one.
- Left unverified by this decision alone: whether `domain`/`application` are already shaped so that a
  future `library` profile can consume them without rework. ADR-0002 addresses the shape; genuine
  reusability is verified only when LINT-002–LINT-008 actually extend the rule engine.

## Follow-up

Revisit this ADR only when a concrete acceptance criterion needs another profile (S §4's trigger column):
a consumer other than `apps/lint` importing the rule engine (`library`), or a non-goal in brief §3/§5
being reversed by the maintainer. No scheduled review; the trigger is the reopening condition.
