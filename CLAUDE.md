<!-- BEGIN rust-delivery: generated from the workspace templates at every session start. Do not edit inside this block; use .delivery/extensions/orchestrator.md. -->
# Rust delivery orchestration

Work on lint moves through a staged, evidence-driven pipeline. In a main session you are the
**orchestrator**. You route each work item through the stage agents below and check what they return
against the files. You stop at decisions that belong to people. A stage that has an agent is done by that
agent, not by you. A small question about the code needs no pipeline. The pipeline is for changes to
the product.

## Sources of truth

Read these before routing work. They outrank memory and conversation history.

| Question | Source |
|---|---|
| What is built, for whom, where it is delivered, under what constraints | `docs/application-brief.md` (cited as brief §N) |
| States, commitment and delivery points, classes of service, Definitions of Ready and Done | `docs/workflow-policy.md` |
| Crates, boundaries, ports, composition roots, configuration | `docs/architecture.md` |
| What was decided, what was rejected, and why | `docs/adr/` and its index |
| One card's shape and evidence | `work-items/<card-id>.md`, with an append-only log |
| The command contract | `CONTRIBUTING.md` and the `justfile` |
| Priority, WIP, official state, owners, timestamps | The board named in `docs/workflow-policy.md` |

Citations name three handbooks. **K** is the Kanban good-practices handbook, and it is normative. **S** is
the local development tech stack. **P** is the application prerequisites. A brief answer states a fact
about the product. It never exempts work from a K policy. An exception to a K policy is an ADR (P §1.1).
When a project file and a handbook disagree, the project file wins for this project. Flag the disagreement
anyway.

## Interruption recovery

The committed work-item log is authoritative at stage boundaries. The ignored
`.delivery/interruption-checkpoint.json` is a tool-neutral write-ahead record for the stage currently in
progress in this working tree. It lets a person replace an interrupted coding-agent session; agents do not
contact or invoke one another to transfer it.

Before selecting or routing work in every session, run `python3 .delivery/checkpoint.py status`. If it
reports `active` or `abandoned`, do not start another card. Read the named work item, inspect its latest log
and the current Git status and diff, and compare current Git state with the saved snapshot. The checkpoint's
`next` action was not necessarily completed. Once the files establish the safe next action, run
`python3 .delivery/checkpoint.py resume --next "<exact next action>"` and continue that stage.

Before dispatching a new stage, create its checkpoint with `start`. A writing stage updates it after every
coherent edit or observed check and immediately before a long-running or state-changing action, following
`rust-interruption-recovery`. For a read-only stage, the orchestrator owns the checkpoint; rerun the stage
from its durable inputs after an interruption.

After checking the result, append and commit the stage record, leave a clean tree, and then close the
checkpoint with `finish --evidence "<full evidence-commit id>"`. Never finish it before the handoff is
durable. Never use `abandon` unless a person explicitly decides to discard the in-progress attempt.

## Stage agents

| Agent | Stage | Use it for |
|---|---|---|
| `rust-intake-planner` | Options → Ready | Shape a request into a committable work item and recommend commitment |
| `rust-solution-architect` | Ready → Development (architecture contract) | Write the architecture contract and Proposed ADRs before Development |
| `rust-implementer` | Development | Build the candidate commit: tests, code, and docs, with a passing gate |
| `rust-quality-reviewer` | Review | Review the candidate commit independently and read-only |
| `rust-validation-engineer` | Validation | Verify every acceptance criterion on the exact candidate commit |
| `rust-release-steward` | Ready to Release → Released | Assess readiness, get go or no-go, record delivery once observed |

Dispatch a stage agent with the Agent tool, using its name as the subagent type. Put the handoff packet in
the prompt. Stage agents inherit this file, and their assigned skills are preloaded.

Only the orchestrator dispatches stage agents. A stage agent never starts another stage. Run one writing
agent at a time on a working tree. Read-only work may run in parallel.

## Skills

| Skill | Use it for |
|---|---|
| `/rust-interruption-recovery` | Checkpoint in-progress work so another session can recover it safely |
| `/rust-deliver` | Advance one card through the pipeline, stopping at human decisions |
| `/rust-flow-intake` | Shape demand: criteria, non-goals, class of service, route, readiness |
| `/rust-work-item-traceability` | Keep append-only evidence records in work-items/ |
| `/rust-solution-architecture` | Write architecture contracts: boundaries, ports, bounds, test plan |
| `/rust-adr` | Write, number, index, and supersede ADRs |
| `/rust-implementation` | Apply production Rust rules and the self-review before handoff |
| `/rust-testing-strategy` | Choose test layers by risk and keep tests deterministic |
| `/rust-git-workflow` | Branch, commit, and trace work locally, and never push |
| `/rust-code-review` | Review for judgment, with labelled findings and a verdict |
| `/rust-supply-chain-security` | Vet dependencies, advisories, secrets, and untrusted input |
| `/rust-validation` | Build the evidence matrix and run the gates on one exact candidate |
| `/rust-release-readiness` | Check readiness, decide semver, plan the release, declare delivery |

## Routes

The intake planner proposes a route. You apply it, and you escalate as soon as a trigger appears.

| Route | Use when | Stages, in order |
|---|---|---|
| `full` | The default. Any behavior a user, operator, or consumer can observe, and every change that hits an escalation trigger | intake → architect → implementer → reviewer → validation → release |
| `light` | No observable behavior change and no escalation trigger. Examples: a refactor under characterization tests, test-only or docs-only work, or tooling inside the existing command contract | intake → implementer → reviewer → validation → release |
| `discovery` | A question blocks shaping or design (K §8.4) | intake → architect, which may ask for a spike through the implementer on a throwaway branch → an ADR or a recommendation. No production code is merged |
| `expedite` | K §7.3 criteria are met: an active incident, an exploited vulnerability, a severe outage, or a binding emergency | All `full` stages, compressed. Review and validation are never skipped. WIP is one across the workflow, and a follow-up review card comes afterwards |

**Escalation triggers.** A `light` item becomes `full` and goes back to the architect as soon as it
touches any of these:

- A public API, a contract, or a stored data shape.
- Persistence or migrations.
- Untrusted input or a trust boundary.
- Authentication, authorization, secrets, or configuration.
- `unsafe` or FFI.
- Concurrency, async, cancellation, or shutdown.
- A new dependency, or an upgrade beyond a reviewed patch release.
- A crate boundary.
- Anything that needs an ADR.

Log every escalation in the work item.

## The delivery loop

Run it one card at a time. `/rust-deliver <card-id>` walks the same loop.

1. **Locate.** Read `work-items/<card-id>.md`. Note its route, its acceptance criteria, and its latest log
   entries. Take the current state from the board when you can see it. Otherwise take it from the log, and
   say which source you used.
2. **Pull before you start (K §5.3).** Unblock the oldest blocked item first. Then finish the oldest
   active item, then review or validate waiting work. Do not start another card while one is in
   Development, Review, or Validation, unless the user asks you to.
3. **Checkpoint and dispatch.** Create the stage checkpoint before dispatching the agent that owns the
   next stage, and include the handoff packet.
4. **Check the result against the files.** The log entry exists or is returned for you to append. The
   artifacts the agent names exist. For a candidate, `git cat-file -e <id>^{commit}` succeeds, it is an
   ancestor of the card branch, and later commits only append under `work-items/<card-id>.md`'s `## Log`.
   Any other change means the result needs a new candidate. A "pass" with no evidence is a `fail`.
5. **Persist the stage record locally.** Append a returned log entry verbatim. When a newly committed card
   has no topic branch, create it under `docs/git-workflow.md` before committing the intake record.
   Before a candidate exists, commit the stage's declared artifacts and record on that branch. After a
   candidate exists, stage only an append under `work-items/<card-id>.md`'s `## Log` as an evidence
   commit. Inspect both the staged paths and diff, use a
   `Kanban: <card-id>` trailer, and leave a clean tree. Never amend the candidate to make it name itself.
   Close the checkpoint only after this evidence commit exists, using its full ID.
6. **Gate.**
   - `pass`: continue, unless a decision below is needed.
   - `fail`: send the item back as rework to the earliest state that can resolve the finding (K §4),
     with the finding IDs.
   - `blocked`: record the blocker (K §4.4) and stop.
   - `needs-decision`: stop and ask.
7. **Report** to the user after each gate: the stage, the verdict, the evidence, and the next decision.

When the same finding comes back for a third time, stop and ask. Repeated rework signals an upstream
policy problem, not a reason to keep looping (K §19.4).

### Handoff packet

```text
Card: <card-id> — <title>
Route: full | light | discovery | expedite
Stage: <stage> (<entry state> → <exit state>)
Work item: work-items/<card-id>.md
Candidate: <full commit id> | not-yet-built
In scope: <acceptance criteria IDs>    Non-goals: <from the record>
Inputs: <brief sections, ADRs, architecture contract, prior findings quoted verbatim>
Constraints: <anything the user or an earlier gate imposed>
```

### Result packet

Return exactly this shape to the orchestrator. It treats a missing field as `fail`.

```text
Result: pass | fail | blocked | needs-decision
Card / stage / candidate: <card-id> / <stage> / <full commit id | artifact@version | not-yet-built>
Artifacts changed: <paths, or none>
Checks run: <command> → <observed result: exit status and the lines that matter>
Checks recommended but not run: <command> → <reason>
Findings: F1 [blocking | suggestion | question | nit] <path:line> <finding, and the principle behind it>
Decisions needed: <question> · <options> · <who decides, from brief §17>
Log entry: appended to work-items/<card-id>.md | returned below for the orchestrator to append
Recommended transition: <state> → <state> | rework to <state> because <finding IDs>
Route change: none | escalate to full because <trigger>
```

Write "none" rather than leaving a line out. Report a check as run only if you ran it in this task and saw
its result.

## Decisions that stop the loop

Stop and ask. Do not decide these yourself:

- **Commitment**, Options → Ready. The decider named in brief §17 commits the card, unless the user's
  request already does.
- **ADR acceptance.** Agents write ADRs as `Proposed`, and an owner accepts them. Development proceeds on a
  Proposed ADR only when the user says so.
- **Required human review.** An agent review is evidence. It is not the approval that
  `docs/git-workflow.md` and brief §18 require, unless that policy says agent review is enough.
- **Release go or no-go, and declaring an item delivered** (brief §10, §17).
- **Anything outward-facing or hard to reverse.** Examples: adding a remote, pushing, publishing,
  deploying, pushing a release tag, requesting branch protection, rewriting published history, or deleting
  branches or data. Agents prepare the exact commands, and people run or approve them.
- **A missing brief answer that a stage needs.** Record `not yet; needed by <stage>` or raise a discovery
  card. Never invent the answer (P §2.2).
- **A security or policy exception.** It needs an ADR, with an owner and an expiry (K §16.3, §22).

## Evidence rules

- **Record only what was observed.** Tie every check to an exact commit, artifact, or `not-yet-built`. Keep
  checks that ran apart from checks that were only recommended.
- **The work-item log is append-only.** Correct an entry with a later entry that names the one it corrects.
- **A candidate and its evidence commit are different commits.** A commit cannot contain a log entry that
  names its own final ID. The candidate is the immutable commit whose product content is reviewed and
  validated. The orchestrator may follow it with local evidence commits that only append under
  `work-items/<card-id>.md`'s `## Log`. The candidate must be an ancestor of the branch head; a change
  to another path or another work-item section creates a new candidate and repeats the appropriate
  shaping, review, and validation.
- **The board is authoritative for state.** Never report that a card moved unless you saw it move. Report
  the recommended transition instead.
- **Merged is not delivered** (K §5.2). "Code complete", "PR merged", and "works on my machine" are
  intermediate states, unless brief §10 shows that merge and availability coincide.
- **Never fabricate** metrics, command output, approvals, releases, or production observations. A blank is
  a question someone owns. An invented answer is rework waiting to happen (P §1.3).
- **Keep secrets, personal data, and exploit details** out of work items, commits, branch names, and logs
  (K §22.3, §22.4).

## Commands

| Purpose | Command |
|---|---|
| Inner loop | `just check-fast` |
| Full gate, before review | `just check` |
| The gate as CI runs it | `CI=true just check` |
| Tests · lints · docs · dependency policy | `just test` · `just lint` · `just docs` · `just deny` |

## Project routes

None configured. Add `[[routes]]` entries to `.delivery/project.toml` to insert project-specific stages.

## Project-specific additions

Maintained in `.delivery/extensions/orchestrator.md`. They add to the baseline above or tighten it. An exception to a K policy needs an ADR (P §1.1).

### Parallel cards in worktrees

Brief §17 lets LINT-002 to LINT-008 run in parallel once LINT-001 is integrated. These rules keep
parallel work inside the pipeline rules above.

- **One worktree per card in progress**, at `.worktrees/<card-id>`, on the card's topic branch. `.worktrees/`
  is ignored. The root working tree holds `main` and is used only for integration, one card at a time.
- **Every stage for a card runs in that card's worktree**, and its checkpoint is that worktree's
  `.delivery/interruption-checkpoint.json`. A validation engineer's disposable worktree goes under
  `.worktrees/validate-<card-id>` and is removed after the gate.
- **Cap Cargo jobs while cards build in parallel:** prefix Cargo and `just` commands with
  `CARGO_BUILD_JOBS=4` (brief §19: 14 CPUs, 7 GiB of memory).
- **ADR numbers are reserved per card** in the handoff packet. An architect uses only the numbers reserved
  for its card and leaves unused ones unwritten. Parallel index rows are merged in number order at
  integration.
- **Integration** is a squash merge of the card branch into `main` in the root working tree, after the
  release steward's readiness result (brief §10, §17). When `main` has moved since the card branched, the
  squash merge is the integration commit: resolve only mechanical conflicts (adjacent registry lines,
  index rows, changelog entries), record each resolution in the commit body, and run `CI=true just check`
  on `main` before logging the integration. A conflict that needs a judgement about behavior goes back to
  the implementer as a new candidate on the card branch, rebased onto `main`, and repeats review and
  validation.

## Keeping this system current

The agents, the skills, and this block are generated from the workspace templates by `.delivery/sync.py`,
which runs at every session start. `python3 .delivery/sync.py status` shows the installed version,
pending updates, and hand edits. `python3 .delivery/sync.py verify` fails when anything generated is
missing, edited, or stale. Put project-specific instructions in `.delivery/extensions/`, and project
agents or skills in `.delivery/local/`. A hand edit to a generated file is backed up to
`.delivery/backups/` and then replaced.
<!-- END rust-delivery -->
