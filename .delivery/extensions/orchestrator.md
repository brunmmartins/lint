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
