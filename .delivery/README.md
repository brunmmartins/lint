# `.delivery/`: the project's agent system

This folder connects the project to the **rust-delivery** agent system. That system is the stage agents,
the skills, and the orchestrator for Claude Code and Codex. The workspace `templates/` folder is their
source. The sync engine renders them into this project and applies what this folder adds.

This README is generated. Edit the files it describes, not this one.

## What is generated, and where

| Generated for | Claude Code | Codex |
|---|---|---|
| Orchestrator (a managed block; text outside the block stays yours) | `CLAUDE.md` | `AGENTS.md` |
| Stage agents | `.claude/agents/<name>.md` | `.codex/agents/<name>.toml` |
| Skills | `.claude/skills/<name>/` | `.agents/skills/<name>/` |
| SessionStart hook that keeps all of it current | `.claude/settings.json` | `.codex/hooks.json` |

Commit the generated files. They let a clone work without the workspace templates. When the templates are
not found, the hook does nothing, and the committed files are used as they are.

## What belongs to the project

| Path | Purpose | Overwritten by sync? |
|---|---|---|
| `project.toml` | Name, commands, tools, sync mode, per-agent and per-skill settings, project routes | Never |
| `extensions/orchestrator.md` | Project rules for the orchestrator | Never |
| `extensions/agents/<name>.md` | Project rules for one template agent | Never |
| `extensions/skills/<name>.md` | Project rules for one template skill | Never |
| `local/agents/<name>.md` | A project-only agent, generated for both tools | Never |
| `local/skills/<name>/SKILL.md` | A project-only skill with its supporting files, generated for both tools | Never |
| `sync.py`, `checkpoint.py`, `README.md`, `.gitignore` | Engines and documentation | Yes |
| `sync-lock.json` | What the last sync wrote, with hashes. Commit it | Yes |
| `interruption-checkpoint.json` | Local active-stage recovery state. Ignore it | Written by delivery sessions |
| `backups/` | Hand edits to generated files, saved before they were replaced. Ignored by Git | Written, never pruned |

## Extending the system

Choose the smallest extension that does the job.

1. **Change a value.** Commands, the card ID scheme, and a tool's model or sandbox all live in
   `project.toml`. For example, set `[commands] check = "cargo test --workspace"` when the project has no
   `justfile`.
2. **Add rules to an existing agent or skill.** Create `extensions/agents/rust-implementer.md`. Its text
   is inserted into both tools' versions of that agent, under "Project-specific additions". Extensions
   add rules or tighten them. An extension that loosens a K policy must cite the ADR that allows it
   (P §1.1).
3. **Add a stage.** Write a project agent, then add a `[[routes]]` entry to `project.toml`. The
   orchestrator lists the route and dispatches the agent when the trigger applies.
4. **Add a capability.** Run `python3 .delivery/sync.py new skill <name>`. That creates
   `local/skills/<name>/SKILL.md`. Give an agent the skill with `extra_skills` in `project.toml`.
5. **Replace a template component.** Set `enabled = false` for it in `project.toml`. Then create a local
   one with the same name. The sync refuses a local component that shadows an enabled template one.

### Source format

Template and local components use the same format: TOML frontmatter between `+++` lines, then a Markdown
body.

```markdown
+++
name = "docchain-consensus-reviewer"
description = "Reviews changes to consensus-path modules for determinism and activation-height safety."
stage = "Review (consensus path)"
skills = ["rust-code-review", "docchain-canonical-form"]

[claude]
tools = ["Read", "Grep", "Glob", "Bash"]
model = "inherit"

[codex]
sandbox_mode = "read-only"
+++

# Consensus reviewer

Mission, stage boundary, procedure, and what it returns.

{{agent_skills}}

## Return

{{partials.result_packet}}
```

Bodies, extensions, and route fields can use these template forms:

| Form | Renders as |
|---|---|
| `{{project.name}}`, `{{project.work_item_scheme}}` | Values from `[project]` |
| `{{commands.check}}` and the other command keys | Values from `[commands]` |
| `{{vars.<key>}}` | Values from `[vars]` |
| `{{tool.name}}`, `{{tool.instructions}}`, `{{tool.skills_dir}}`, `{{tool.skill_prefix}}` | The tool being rendered |
| `{{agent_skills}}` | In an agent: its skills, preloaded for Claude Code and listed by path for Codex |
| `{{partials.result_packet}}`, `{{partials.evidence_rules}}` | Shared fragments from the templates |
| A `{{#claude}}` … `{{/claude}}` or `{{#codex}}` … `{{/codex}}` section | Text included for that tool only |

An unknown variable stops the sync before anything is written. The session-start output names the file
and line.

## Commands

```bash
python3 .delivery/sync.py status              # installed and available versions, pending changes, hand edits
python3 .delivery/sync.py sync [--dry-run]    # apply now; also adopts updates in pinned mode
python3 .delivery/sync.py verify              # exit 1 unless complete, unedited, and current
python3 .delivery/sync.py new agent|skill <name>
```

## Recover interrupted work

`checkpoint.py` keeps one local write-ahead record for the delivery stage active in this Git working tree.
The SessionStart hook reports an unfinished or abandoned record. It is tool-neutral: a person may stop one
coding-agent process and open the same working tree with another; the agents do not transfer or invoke one
another.

Use `status` before selecting work, `start` before a stage, `update` after each coherent unit and before a
long-running action, `resume` only after inspecting the saved and current Git state, and `finish` only after
the stage's work-item evidence is committed and the tree is clean:

```bash
python3 .delivery/checkpoint.py status
python3 .delivery/checkpoint.py start --card "<id>" --stage "<stage>" --objective "<outcome>" --next "<action>"
python3 .delivery/checkpoint.py update --completed "<observed fact>" --next "<action>"
python3 .delivery/checkpoint.py resume --next "<verified next action>"
python3 .delivery/checkpoint.py finish --evidence "<full evidence-commit id>"
```

The ignored `interruption-checkpoint.json` follows this working tree and may describe uncommitted work. It
does not travel to another clone. A cross-machine handoff still requires the normal committed work-item log.
Never put secrets, personal data, credentials, or restricted vulnerability details in a checkpoint.

The engine finds the templates in this order:

1. The `RUST_DELIVERY_TEMPLATES` environment variable.
2. The `source` recorded in `sync-lock.json`.
3. A `templates/` folder in this directory or any parent.

## When something looks wrong

- **A session reports "sync skipped".** Run `status`. The message names the file to fix. Nothing was
  written.
- **Codex does not run the hook.** Trust the project's hook once with `/hooks`. Also check that the project
  itself is trusted.
- **New agents or skills do not appear.** A directory created during a session is only watched by sessions
  that start after it. Start a new session.
- **A generated file was edited by hand.** The next sync saves it under `backups/` and replaces it. Move
  the lasting part into `extensions/`.
