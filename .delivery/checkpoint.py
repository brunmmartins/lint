#!/usr/bin/env python3
"""Maintain a tool-neutral, crash-safe checkpoint for one delivery stage.

The checkpoint is local to a working tree and ignored by Git. Durable stage evidence still belongs in
the work item's append-only log and in commits. This file closes the gap between the last durable handoff
and an arbitrary process, terminal, or coding-agent interruption.

Commands:
  start    create a write-ahead checkpoint before a stage starts
  update   record completed work and the exact next action
  resume   adopt an unfinished checkpoint in a new session
  finish   close a checkpoint after its evidence commit exists and the tree is clean
  abandon  retain a deliberately abandoned checkpoint and its reason
  status   show the record and compare its Git snapshot with the current tree

Exit status: 0 on success, 1 on an invalid transition or missing prerequisite, 2 on bad usage.
Needs Python 3.11 or newer and nothing outside the standard library.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path


SCHEMA = 1
DELIVERY_DIR = ".delivery"
CHECKPOINT_NAME = "interruption-checkpoint.json"
MAX_FIELD = 4000
MAX_ENTRY = 1200
MAX_ENTRIES = 200
FULL_COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")


class CheckpointError(Exception):
    """A problem that must leave the existing checkpoint untouched."""


def utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds")


def resolve_project(argument: str | None) -> Path:
    if argument:
        project = Path(argument).resolve()
    else:
        own = Path(__file__).resolve()
        project = own.parent.parent if own.parent.name == DELIVERY_DIR else Path.cwd().resolve()
    if not project.is_dir():
        raise CheckpointError(f"{project}: project directory does not exist")
    return project


def checkpoint_path(project: Path) -> Path:
    return project / DELIVERY_DIR / CHECKPOINT_NAME


def bounded(value: str, label: str, maximum: int = MAX_FIELD) -> str:
    value = value.strip()
    if not value:
        raise CheckpointError(f"{label} must not be blank")
    if len(value) > maximum:
        raise CheckpointError(f"{label} is longer than {maximum} characters")
    return value


def atomic_write(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(payload)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
        try:
            directory = os.open(path.parent, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        except OSError:
            return
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        if temporary.exists():
            temporary.unlink()


def read_checkpoint(project: Path, required: bool = True) -> dict | None:
    path = checkpoint_path(project)
    if not path.is_file():
        if required:
            raise CheckpointError("no interruption checkpoint exists")
        return None
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise CheckpointError(f"{path}: unreadable checkpoint ({error})") from error
    if not isinstance(value, dict) or value.get("schema") != SCHEMA:
        raise CheckpointError(f"{path}: unsupported or invalid checkpoint schema")
    required_strings = ("status", "card", "stage", "objective", "next", "started_at", "updated_at")
    if any(not isinstance(value.get(key), str) or not value[key] for key in required_strings):
        raise CheckpointError(f"{path}: checkpoint is missing a required string field")
    if value["status"] not in ("active", "finished", "abandoned"):
        raise CheckpointError(f"{path}: invalid checkpoint status {value['status']!r}")
    for key in ("completed", "checks", "resume_events"):
        if not isinstance(value.get(key), list) or len(value[key]) > MAX_ENTRIES:
            raise CheckpointError(f"{path}: {key} must be a bounded array")
    for key in ("completed", "checks"):
        if any(
            not isinstance(entry, dict)
            or not isinstance(entry.get("at"), str)
            or not isinstance(entry.get("text"), str)
            for entry in value[key]
        ):
            raise CheckpointError(f"{path}: {key} contains an invalid entry")
    snapshots = [value.get("started_git"), value.get("git")]
    snapshots.extend(entry.get("git") for entry in value["resume_events"] if isinstance(entry, dict))
    for snapshot in snapshots:
        if (
            not isinstance(snapshot, dict)
            or not isinstance(snapshot.get("branch"), str)
            or not (snapshot.get("head") is None or isinstance(snapshot.get("head"), str))
            or not isinstance(snapshot.get("dirty"), list)
            or any(not isinstance(item, str) for item in snapshot["dirty"])
        ):
            raise CheckpointError(f"{path}: checkpoint contains an invalid Git snapshot")
    return value


def run_git(project: Path, *arguments: str, text: bool = True) -> subprocess.CompletedProcess:
    try:
        return subprocess.run(
            ["git", "-C", str(project), *arguments],
            capture_output=True,
            text=text,
            check=False,
            timeout=15,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        raise CheckpointError(f"could not inspect Git state ({error})") from error


def git_snapshot(project: Path) -> dict:
    root = run_git(project, "rev-parse", "--show-toplevel")
    if root.returncode != 0:
        detail = root.stderr.strip() or "not a Git working tree"
        raise CheckpointError(f"recovery checkpoints require a Git working tree: {detail}")
    git_root = Path(root.stdout.strip()).resolve()
    if git_root != project.resolve():
        raise CheckpointError(f"project {project} is inside Git worktree {git_root}; run from its root")

    head_result = run_git(project, "rev-parse", "--verify", "HEAD")
    head = head_result.stdout.strip() if head_result.returncode == 0 else None
    branch_result = run_git(project, "symbolic-ref", "--quiet", "--short", "HEAD")
    branch = branch_result.stdout.strip() if branch_result.returncode == 0 else "detached"
    status = run_git(project, "status", "--porcelain=v1", "-z", "--untracked-files=all", text=False)
    if status.returncode != 0:
        detail = status.stderr.decode(errors="replace").strip() or "git status failed"
        raise CheckpointError(f"could not capture working-tree state: {detail}")
    dirty = [item.decode("utf-8", errors="surrogateescape") for item in status.stdout.split(b"\0") if item]
    return {"branch": branch, "head": head, "dirty": dirty}


def append_entries(state: dict, key: str, values: list[str]) -> None:
    entries = state[key]
    additions = [{"at": utc_now(), "text": bounded(value, key, MAX_ENTRY)} for value in values]
    if len(entries) + len(additions) > MAX_ENTRIES:
        raise CheckpointError(
            f"{key} has more than {MAX_ENTRIES} entries; persist a stage handoff before continuing"
        )
    entries.extend(additions)


def require_status(state: dict, *allowed: str) -> None:
    if state["status"] not in allowed:
        choices = " or ".join(allowed)
        raise CheckpointError(f"checkpoint is {state['status']}; this command requires {choices}")


def checkpoint_start(args: argparse.Namespace) -> int:
    project = resolve_project(args.project)
    existing = read_checkpoint(project, required=False)
    if existing and existing["status"] == "active":
        raise CheckpointError(
            f"unfinished checkpoint for {existing['card']} / {existing['stage']}; resume it before starting another"
        )
    current_git = git_snapshot(project)
    if existing and existing["status"] == "abandoned" and current_git["dirty"]:
        raise CheckpointError("the abandoned checkpoint still has a dirty tree; recover or clean it before replacing it")
    now = utc_now()
    state = {
        "schema": SCHEMA,
        "status": "active",
        "card": bounded(args.card, "card"),
        "stage": bounded(args.stage, "stage"),
        "objective": bounded(args.objective, "objective"),
        "next": bounded(args.next, "next"),
        "candidate": bounded(args.candidate, "candidate") if args.candidate else None,
        "started_at": now,
        "updated_at": now,
        "started_git": current_git,
        "git": current_git,
        "completed": [],
        "checks": [],
        "resume_events": [],
    }
    atomic_write(checkpoint_path(project), state)
    print(f"checkpoint started: {state['card']} / {state['stage']}")
    print(f"next: {state['next']}")
    return 0


def checkpoint_update(args: argparse.Namespace) -> int:
    project = resolve_project(args.project)
    state = read_checkpoint(project)
    require_status(state, "active")
    append_entries(state, "completed", args.completed)
    append_entries(state, "checks", args.check)
    if args.candidate:
        state["candidate"] = bounded(args.candidate, "candidate")
    state["next"] = bounded(args.next, "next")
    state["updated_at"] = utc_now()
    state["git"] = git_snapshot(project)
    atomic_write(checkpoint_path(project), state)
    print(f"checkpoint updated: {state['card']} / {state['stage']}")
    print(f"next: {state['next']}")
    return 0


def checkpoint_resume(args: argparse.Namespace) -> int:
    project = resolve_project(args.project)
    state = read_checkpoint(project)
    require_status(state, "active", "abandoned")
    previous_status = state["status"]
    current_git = git_snapshot(project)
    if len(state["resume_events"]) >= MAX_ENTRIES:
        raise CheckpointError("too many resume events; persist a stage handoff before continuing")
    state["resume_events"].append({"at": utc_now(), "from_status": previous_status, "git": current_git})
    state["status"] = "active"
    if args.next:
        state["next"] = bounded(args.next, "next")
    state["updated_at"] = utc_now()
    state["git"] = current_git
    atomic_write(checkpoint_path(project), state)
    print(f"checkpoint resumed: {state['card']} / {state['stage']}")
    print(f"next: {state['next']}")
    return 0


def require_evidence_commit(project: Path, commit: str) -> None:
    if not FULL_COMMIT_RE.fullmatch(commit):
        raise CheckpointError("evidence must be a full 40-character lowercase commit ID")
    exists = run_git(project, "cat-file", "-e", f"{commit}^{{commit}}")
    if exists.returncode != 0:
        raise CheckpointError(f"evidence commit {commit} does not exist")
    ancestor = run_git(project, "merge-base", "--is-ancestor", commit, "HEAD")
    if ancestor.returncode != 0:
        raise CheckpointError(f"evidence commit {commit} is not an ancestor of HEAD")


def checkpoint_finish(args: argparse.Namespace) -> int:
    project = resolve_project(args.project)
    state = read_checkpoint(project)
    require_status(state, "active")
    current_git = git_snapshot(project)
    if current_git["dirty"]:
        summary = ", ".join(current_git["dirty"][:5])
        raise CheckpointError(f"working tree is not clean ({summary}); persist the handoff before finishing")
    evidence = bounded(args.evidence, "evidence commit")
    require_evidence_commit(project, evidence)
    now = utc_now()
    state.update({
        "status": "finished",
        "updated_at": now,
        "finished_at": now,
        "evidence_commit": evidence,
        "git": current_git,
        "next": "Read the committed work-item log before selecting the next stage.",
    })
    atomic_write(checkpoint_path(project), state)
    print(f"checkpoint finished: {state['card']} / {state['stage']} at {evidence}")
    return 0


def checkpoint_abandon(args: argparse.Namespace) -> int:
    project = resolve_project(args.project)
    state = read_checkpoint(project)
    require_status(state, "active")
    now = utc_now()
    state.update({
        "status": "abandoned",
        "updated_at": now,
        "abandoned_at": now,
        "abandon_reason": bounded(args.reason, "reason"),
        "git": git_snapshot(project),
        "next": "A person must decide whether to resume or clean the retained work.",
    })
    atomic_write(checkpoint_path(project), state)
    print(f"checkpoint abandoned but retained: {state['card']} / {state['stage']}")
    return 0


def git_label(snapshot: dict) -> str:
    head = snapshot.get("head") or "no-commit"
    dirty = snapshot.get("dirty", [])
    return f"{snapshot.get('branch', 'unknown')} @ {head}; {len(dirty)} dirty path(s)"


def checkpoint_status(args: argparse.Namespace) -> int:
    project = resolve_project(args.project)
    state = read_checkpoint(project, required=False)
    if state is None:
        if not args.hook:
            print("no interruption checkpoint")
        return 0
    if args.json:
        print(json.dumps(state, indent=2, sort_keys=True))
        return 0
    current_git = git_snapshot(project)
    if args.hook:
        if state["status"] == "finished":
            return 0
        label = "unfinished" if state["status"] == "active" else "abandoned"
        print(f"rust-delivery: {label} interruption checkpoint detected; do not start another card.")
        print(f"  card/stage  {state['card']} / {state['stage']}")
        print(f"  updated     {state['updated_at']}")
        print(f"  next        {state['next']}")
        print(f"  checkpoint  {git_label(state['git'])}")
        print(f"  current     {git_label(current_git)}")
        print("  recover     inspect the work item and Git diff, then run "
              "python3 .delivery/checkpoint.py resume")
        return 0

    print(f"status:      {state['status']}")
    print(f"card/stage:  {state['card']} / {state['stage']}")
    print(f"objective:   {state['objective']}")
    print(f"started:     {state['started_at']}")
    print(f"updated:     {state['updated_at']}")
    print(f"candidate:   {state.get('candidate') or 'not-yet-built'}")
    print(f"checkpoint:  {git_label(state['git'])}")
    print(f"current:     {git_label(current_git)}")
    if state["git"] != current_git:
        print("drift:       Git state differs from the last checkpoint; inspect it before resuming")
    print("completed:")
    for entry in state["completed"]:
        print(f"  - {entry['at']} {entry['text']}")
    if not state["completed"]:
        print("  - none")
    print("checks:")
    for entry in state["checks"]:
        print(f"  - {entry['at']} {entry['text']}")
    if not state["checks"]:
        print("  - none")
    print(f"next:        {state['next']}")
    return 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(prog="checkpoint.py", description=__doc__.split("\n\n")[0])
    result.add_argument("--project", help="project root; defaults to the parent of this installed script")
    commands = result.add_subparsers(dest="command", required=True)

    start = commands.add_parser("start", help="create a write-ahead stage checkpoint")
    start.add_argument("--card", required=True)
    start.add_argument("--stage", required=True)
    start.add_argument("--objective", required=True)
    start.add_argument("--next", required=True)
    start.add_argument("--candidate")

    update = commands.add_parser("update", help="record progress and the next recovery action")
    update.add_argument("--completed", action="append", default=[])
    update.add_argument("--check", action="append", default=[])
    update.add_argument("--candidate")
    update.add_argument("--next", required=True)

    resume = commands.add_parser("resume", help="adopt an unfinished checkpoint")
    resume.add_argument("--next", help="replace the saved next action after inspecting current state")

    finish = commands.add_parser("finish", help="close a checkpoint after its evidence commit")
    finish.add_argument("--evidence", required=True, help="full ID of the committed stage evidence")

    abandon = commands.add_parser("abandon", help="retain an intentionally abandoned checkpoint")
    abandon.add_argument("--reason", required=True)

    status = commands.add_parser("status", help="show saved and current recovery state")
    status.add_argument("--hook", action="store_true", help=argparse.SUPPRESS)
    status.add_argument("--json", action="store_true")
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    handlers = {
        "start": checkpoint_start,
        "update": checkpoint_update,
        "resume": checkpoint_resume,
        "finish": checkpoint_finish,
        "abandon": checkpoint_abandon,
        "status": checkpoint_status,
    }
    try:
        return handlers[args.command](args)
    except CheckpointError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
