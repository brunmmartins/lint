# Work items

Each board card has one durable evidence record, `work-items/<card-id>.md`, started from
[TEMPLATE.md](TEMPLATE.md). Card IDs follow `LINT-NNN`, and they never change. There is no
board yet (brief §17); until one exists, the latest gate result in each record's log stands in for state.

These records complement the board. They are not a second board. The board is authoritative for
priority, WIP, ordering, owners, official state, and timestamps.

## Rules

- **Create the record** when the card is shaped or committed, whichever produces evidence first.
- **Append a dated log entry** at every material event: a handoff, a gate result, a blocker raised or
  cleared, a rework transition, a candidate change, or a release event.
- **Record only what was observed.** Tie every check to an exact commit, artifact, or version, or to
  `not-yet-built`. List checks that actually ran apart from checks that were only recommended.
- **Never rewrite a log entry.** Correct it with a later entry that names the entry it corrects.
- **Persist stage evidence locally.** The orchestrator commits stage-authored records before the next stage.
  After a product candidate exists, an evidence commit may only append under this card's `## Log`. This is
  necessary because a commit cannot contain a log entry naming its own final ID. The immutable candidate
  remains the object reviewed and validated, and it must be an ancestor of the branch head. A later
  change to any other path or work-item section creates a new candidate and repeats review and validation.
- **Decisions go to ADRs.** Record each consequential decision in [`docs/adr/`](../docs/adr/README.md)
  and link it from the record.
- **Never fabricate** metrics, command results, approvals, releases, or production observations.
