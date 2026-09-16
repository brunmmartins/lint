# lint

`lint` is a small Rust command-line tool for checking Markdown files. It
accepts files and directories, walks directories recursively, and reports each
problem with its path, line, column, rule, and message.

The project is pre-release. It currently checks:

| Rule | Check |
|---|---|
| `MD001` | Headings that skip a level, such as level 3 directly under level 1 |
| `MD009` | Trailing spaces and tabs |
| `MD010` | Hard tabs anywhere, including code blocks and code spans |
| `MD012` | More than one consecutive blank line outside code blocks |
| `MD013` | Lines over 80 characters with whitespace after column 80 |
| `MD018` | A line starting with `#` characters that no space follows |
| `MD022` | Headings without a blank line directly above and below them |
| `MD025` | A second level-1 heading in a document that starts with one |
| `MD047` | Files end with exactly one newline |

`MD010` reports one finding per run of tabs. `MD012` ignores blank lines inside
fenced and indented code blocks, and `MD018` does not check code or HTML
blocks.

`MD013` allows unbroken overflow, such as a long URL, and applies the same
80-character limit to headings, code blocks, and tables. Line lengths and
columns count characters, and a tab counts as one character.

Every rule runs with its default settings. There is no configuration file, no
way to enable or disable rules, no comments that switch rules off inside a
file, no JSON output, and no automatic fixing. YAML and TOML front matter is
not recognised, so its lines are checked like any others.

## Build

Install [Rust](https://www.rust-lang.org/tools/install), then build the
workspace:

```bash
rustup toolchain install --no-self-update
cargo build --locked -p lint
```

The optimized binary can be built with `cargo build --release --locked -p lint`.

## Usage

Pass one or more Markdown files or directories:

```bash
cargo run -p lint -- README.md path/to/markdown
```

Directories are searched recursively for files whose extension is `.md` or
`.markdown`, without following directory symlinks. An explicitly named file is
checked regardless of its extension.

An explicitly named path must be a regular file, or a symlink to one. Named
pipes, devices, process substitutions such as `<(cmd)`, and `/dev/stdin` fed
from a pipe are rejected without being read, with exit status `2`. Inside a
walked directory, such entries are skipped. A file larger than 10 MiB is
rejected.

Findings are written to standard output in this form, each file's findings as
soon as that file has been checked:

```text
path/to/file.md:4:12 MD009 trailing whitespace
```

Errors are written to standard error as they occur.

The process exits with status `0` when no findings remain, `1` when findings
are present, and `2` for invalid arguments, file-system errors, or standard
output that cannot be written. When standard output closes early, for example
when piped into `head`, `lint` stops without checking the remaining paths and
exits with status `2`.

## Development

[`just`](https://github.com/casey/just) provides the standard development
commands:

```bash
just bootstrap
just check-fast
just check
```

The equivalent Cargo commands and contribution workflow are documented in
[CONTRIBUTING.md](CONTRIBUTING.md).

## Project structure

| Path | Purpose |
|---|---|
| `apps/lint/` | CLI, file-system adapters, and end-to-end tests |
| `crates/application/` | Orchestration, output format, and exit-code policy |
| `crates/domain/` | Markdown model and lint rules |

## Security

Please report vulnerabilities as described in [SECURITY.md](SECURITY.md).

## License

Licensed under either the [Apache License 2.0](LICENSE-APACHE) or the
[MIT License](LICENSE-MIT), at your option.
