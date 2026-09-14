# lint

A person writing Markdown, or a CI job, runs one command over files or directories and gets every style
problem reported with its location and rule, and can fix the safe ones automatically.

- **Status:** Pre-release. Scaffolded on 2026-09-14; the first slice, LINT-001, is in progress. There is no
  versioned release.
- **Stack profile:** `cli` (the decision record is not yet written; ADR-0001, LINT-001)
- **Support:** GitHub issues on brunmmartins/lint, once the maintainer creates the repository
- **Security reports:** [SECURITY.md](SECURITY.md)

## What this is, and what it is not

The [application brief](docs/application-brief.md) holds the outcome, the non-goals, who it is for, and
where it is delivered. It describes what is true now, so edit it when an answer changes.

## Quick start

Prerequisites and the first-run path are in [docs/local-development.md](docs/local-development.md).

```bash
just bootstrap
just check-fast
```

If `just` is not installed, [CONTRIBUTING.md](CONTRIBUTING.md#commands) lists the commands behind each recipe.

## Use

Not usable yet; the first delivering slice is LINT-001 (brief §5).

## Repository map

| Path | Holds |
|---|---|
| [`docs/application-brief.md`](docs/application-brief.md) | What is built, for whom, where it is delivered, under what constraints |
| [`docs/architecture.md`](docs/architecture.md) | Boundaries, crates, and composition roots as they are now |
| [`docs/adr/`](docs/adr/README.md) | Decisions: what was chosen, what was rejected, and why |
| [`docs/workflow-policy.md`](docs/workflow-policy.md) | Commitment and delivery points, Definition of Ready, Definition of Done |
| [`work-items/`](work-items/README.md) | One evidence record per board card |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Commands, branches, commits, and review |
| [`docs/git-workflow.md`](docs/git-workflow.md) | Branching, integration, signing, and recovery policy |

## Citations

References like **K §5.2**, **S §16.1**, and **P §10** point to the method handbooks this project was
scaffolded under:

- **K**: `RUST_KANBAN_GOOD_PRACTICES.md`, which sets flow and engineering policy.
- **S**: `RUST_LOCAL_DEVELOPMENT_TECH_STACK.md`, one stack that conforms to K.
- **P**: `RUST_APPLICATION_PREREQUISITES.md`, the template the brief was filled from.

**Brief §N** means section N of [`docs/application-brief.md`](docs/application-brief.md).
