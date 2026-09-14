# Git workflow

This is the Git policy for lint, condensed from S §7. The choices below are recorded in brief
§18 as well. If one changes, change both documents in the same commit.

## Default branch and remote

- `main` stays buildable and releasable. Changes reach it through review, and releases are
  tagged from it (S §7.4).
- Protection, required checks, and approvals: the required check is `CI=true just check` on the exact
  commit, run locally. Agent review and validation gate local integration into `main` (brief §10, §17);
  the maintainer's review is required before any push. No CODEOWNERS. Branch protection is not configured.
- The project was scaffolded without a remote. A maintainer adds the remote and branch protection. No
  agent adds a remote, pushes, or requests protection.

## Branches

Branch from an up-to-date `main`. Keep one card per branch, and measure a branch's life in
hours or days (S §7.5). An unmerged branch is work in progress whether or not a card shows it (K §20.5).

| Prefix | Purpose |
|---|---|
| `feature/` | User or system capability |
| `fix/` | Defect correction |
| `security/` | Restricted security remediation |
| `refactor/` | Behavior-preserving structural change |
| `docs/` | Documentation-only change |
| `chore/` | Tooling or routine maintenance |

Name branches `<prefix>/<card-id>-<short-slug>`, where card IDs follow `LINT-NNN`. Keep
vulnerability details and customer data out of branch names.

## Commits

- Each commit is one coherent change, buildable where practical, and safe to revert (S §7.8).
- The subject says what the commit does, in the imperative. The body says why, including any
  non-obvious tradeoff.
- End with a `Kanban: <card-id>` trailer. Call out breaking behavior, migrations, and operational effects.
- Never include secrets, personal data, or private incident details.
- Before committing, inspect `git status --short` and `git diff --staged`. Commit only what belongs to
  the card.

## Integrating

- Topic-branch update policy: rebase onto `main` while the branch is unpublished (S §7.10).
- Merge strategy: squash merge, so one card becomes one revertible commit on `main`. The squash
  message keeps the card ID and the motivation (S §7.12)
- Never rewrite `main`. To rewrite your own pushed topic branch, push with
  `--force-with-lease --force-if-includes`, and do not run `git fetch` between the rewrite and the push
  (S §7.10).
- Undo a shared change with `git revert <commit>` (S §7.26).
- A merge records the change. It does not deliver the card unless the delivery point in
  [workflow-policy.md](workflow-policy.md) is the merge.

## Signing and tags

- Commit and tag signing: not required. There is no provenance requirement and no verification path,
  so signing would be decoration (S §7.21).
- Release tags are annotated, created only from `main`, and never moved or replaced
  (S §7.21).

## Lockfile, generated files, and migrations

- `Cargo.lock`: committed, because the workspace produces a binary (S §19.2). Review lockfile changes.
- A migration is immutable once shared (S §7.16).
- Commit generated output only together with its source and its deterministic generation command.

## Traceability

```text
card → branch → review → merge commit → release tag → artifact built from that commit
```

Every released artifact names its full commit ID (S §7.22). Git activity is evidence of change, never a
measure of an individual (S §7.30).

## Recovery

Inspect before acting: `git status`, `git diff`, `git log --oneline --graph --all`, `git reflog`.
`git reset --hard`, `git clean -fd`, and force pushes are not routine cleanup. Before any destructive
step, save valuable work to a temporary branch (S §7.26).
