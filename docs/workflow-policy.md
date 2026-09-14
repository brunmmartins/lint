# Workflow policy

This file sets how work on lint moves from commitment to delivery. It adapts K §4–§7 to this
product, as P §21 requires. There is no board yet (brief §17: `unknown: maintainer, review by
2026-09-28`). Until one exists, the maintainer decides state, priority, and ownership, the latest gate
result in each work-item log stands in for card state, and GitHub's push events to `main` are the delivery
timestamps. This file holds the policies a board will apply.

## Commitment and delivery

| | |
|---|---|
| Commitment point | Options → Ready: the maintainer accepts the item. LINT-001 to LINT-008 were accepted by the maintainer's request of 2026-09-14 |
| Delivery point | The change is on `main` at github.com/brunmmartins/lint, timestamped by GitHub's push event. Merge and availability coincide because a consumer builds `lint` from the repository (brief §10). Local integration into `main` is not delivery |
| Post-release verification required before an item counts as delivered | The commit on GitHub's `main` is the one verified, and from a clean checkout of it `CI=true just check` passes and `lint` behaves as the card's acceptance tests predict |
| Approvals required before release | The maintainer's review before a push. No specialist review is required (brief §15) |
| Who may declare an item delivered | The maintainer |

Each point is one transition that a machine can timestamp (K §4.3). Cycle time runs from the
commitment point to the delivery point. "Code complete", "PR merged", and "works on my machine" are
intermediate states. The exception is when brief §10 shows that merging and customer availability
happen together (K §5.2).

## States

Options → Ready → Development → Review → Validation → Released. `Ready` is the commitment point and
`Released` is the delivery point.

Use one spelling per state everywhere: on the board, in automation, in reports, and in work-item
records (K §4.1). Brief §17 records the WIP limits, the SLE percentile, and the day-counting convention.
Use the same percentile and convention for the SLE, the aging bands, and every forecast (K §5.4, §27.2).

- **Blocked is a flag, not a state.** A blocked card stays in its column and still counts against WIP.
  Record when it became blocked, the cause, who owns unblocking it, and when to escalate next (K §4.4).
- **Rework is a backward transition, not a column.** A failed check returns the item to the earliest
  state that can resolve the finding, and the item's record logs why (K §4).

## Pull policy

When capacity frees up, in this order (K §5.3):

1. Unblock the oldest blocked item.
2. Finish the oldest active item.
3. Review or validate an item that is waiting downstream.
4. Pull the highest-priority eligible item, if its WIP limit permits.

## Classes of service

- **Standard:** most work. Selected at replenishment, and pulled in priority and aging order.
- **Fixed date:** only when missing the date has a material consequence. The card shows the date, the
  consequence, and the latest responsible start.
- **Expedite:** only for immediate, severe cost of delay, such as an active incident, an exploited
  vulnerability, a severe outage, or a binding emergency. WIP is one across the whole workflow. Each
  expedite gets a short review afterwards (K §7.3).
- **Intangible:** real benefit without visible delay cost, such as dependency modernization or
  reliability work. Capacity is replenished for it on purpose (K §7.4).

## Definition of Ready

An item is ready when (K §5.1):

- [ ] The desired outcome and customer are stated.
- [ ] The acceptance criteria are observable and testable.
- [ ] It is small enough to finish within the SLE, or it has been split, or it is a timeboxed discovery item.
- [ ] Known dependencies and risks are visible.
- [ ] Required product, security, privacy, or architecture input is available.
- [ ] A validation approach is identified.
- [ ] Rollout and rollback considerations are proportionate to the risk.
- [ ] No unresolved question blocks useful development.

A change to the stack also needs these (S §25.2):

- [ ] The replacement boundary is identified.
- [ ] Local prerequisites are known.
- [ ] Licensing and supply chain are reviewed.
- [ ] Supported platforms are clear.
- [ ] A proof-of-concept stopping condition is set.

## Definition of Done

An item is done when (K §5.2):

- [ ] The acceptance criteria are satisfied, with evidence logged in `work-items/<card-id>.md`.
- [ ] `just check` passes on the exact candidate commit. That covers formatting, Clippy with warnings
      denied, tests including doc tests, docs with warnings denied, and the dependency policy.
- [ ] Failure paths and negative behavior are tested where they matter.
- [ ] Public APIs and operational behavior are documented, and the changelog is updated where it applies.
- [ ] Required peer review is complete, and specialist review is complete where required.
- [ ] Telemetry, runbooks, migrations, and compatibility implications are handled where relevant.
- [ ] Consequential decisions are recorded as ADRs and indexed.
- [ ] The change has reached the delivery point, and post-release verification passed.
- [ ] Temporary flags, TODOs, and follow-up work are tracked as cards.

A change to the stack also needs these (S §25.3):

- [ ] A clean checkout works through the documented commands.
- [ ] Unaffected tests still need no infrastructure.
- [ ] CI uses the same tool versions and commands.
- [ ] Old dependencies, flags, and documentation are removed, or have removal cards.
- [ ] The relevant ADR is accepted.
