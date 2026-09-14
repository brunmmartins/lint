# Changelog

User-visible changes to lint, with migration steps for anything that breaks compatibility
(K §18.1, §21.2).

There are no versioned releases yet. Each card adds an entry under Unreleased for every user-visible
change. When a version is cut, entries move under a `0.x.y` heading, where a change to the CLI contract
(arguments, rule IDs, configuration keys, output, or exit statuses) bumps the minor component.

## Unreleased

- Added the `lint` command-line tool (LINT-001): `lint <path>...` walks files and directories for
  Markdown files, runs MD009 (`no-trailing-spaces`) and MD047 (`single-trailing-newline`) against
  each one, and prints every finding to stdout as `path:line:col RULE message`. Exit status is `0`
  when no finding remains, `1` when at least one finding remains, and `2` on a usage, path-not-found,
  unreadable-file, oversized-file, or invalid-UTF-8 error (an I/O-class fault takes precedence over a
  finding when both occur in the same run). Directories are walked recursively without following
  symlinks, so a symlink loop cannot cause a hang.
