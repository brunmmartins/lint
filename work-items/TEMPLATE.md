# <card-id>: <short, outcome-oriented title>

> Evidence record. The board is authoritative for state, priority, WIP, owners, and timestamps.
> Everything above **Log** describes the item, and is edited as understanding improves.
> The log is append-only.

## Identity

- Card: `<card-id>` on the board
- Demand type: capability | defect | security | reliability | dependency or toolchain | technical debt | discovery | compliance | developer experience | incident follow-up
- Class of service: standard | fixed date (`<date>`; consequence; latest responsible start) | expedite | intangible
- Customer or affected system:
- Brief sections this item draws on: §

## Outcome

Who benefits, what changes, and why it matters.

## Context

Current behavior, the evidence for the problem, and the constraints.

## Acceptance criteria

| ID | Given / When / Then | Verified by |
|---|---|---|
| AC1 | Given … When … Then … | Test path, manual step, dashboard, or release check |

Cover failure and edge behavior. Include telemetry or documentation when they are part of the outcome
(K §8.2, §17.7). Avoid implementation-only criteria unless the item is an internal refactoring.

## Non-goals

- Scope a reasonable reader would otherwise assume is included.

## Risks and dependencies

- Security and privacy:
- Compatibility and migration:
- External dependencies:
- Operational impact:

## Validation approach

- Automated tests:
- Manual or environment verification:
- Performance or security validation:

## Flow and delivery plan

- Delivery point: as in `docs/workflow-policy.md`, or why this item differs
- Rollout:
- Rollback or forward fix:
- Post-release verification:

## Decisions and ADRs

| ADR | Decision | Status |
|---|---|---|

## Log

Append-only. Oldest first. Add one entry per material event, using the shape below.

### YYYY-MM-DD: <event summary> (<stage or role>)

- Event: handoff | gate result: pass or fail | blocker raised | blocker cleared | rework to `<state>` | candidate change | release event | correction of the YYYY-MM-DD entry
- Candidate: `<full commit id>` | `<artifact>@<version>` | `not-yet-built`
- Checks run: `<command>` gave <observed result>
- Checks recommended but not run:
- Evidence: <paths, links, short excerpts>
- Next: <owner or stage>, <action>

For a blocker, also record (K §30.6):

- Blocked since:
- Condition:
- Impact on delivery:
- Unblock action and owner:
- Next escalation:
- External dependency or contact:
