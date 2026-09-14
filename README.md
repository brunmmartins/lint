# lint

`lint` is a small Rust command-line tool for checking Markdown files. It accepts files and directories,
walks directories recursively, and reports each problem with its path, line, column, rule, and message.

The project is pre-release. It currently checks:

| Rule | Check |
|---|---|
| `MD009` | Trailing spaces and tabs |
| `MD047` | Files end with exactly one newline |

## Build

Install [Rust](https://www.rust-lang.org/tools/install), then build the workspace:

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

Directories are searched recursively for files whose extension is `.md` or `.markdown`, without
following directory symlinks. An explicitly named file is checked regardless of its extension.

Findings are written to standard output in this form:

```text
path/to/file.md:4:12 MD009 trailing whitespace
```

The process exits with status `0` when no findings remain, `1` when findings are present, and `2` for
invalid arguments or file-system errors.

## Development

[`just`](https://github.com/casey/just) provides the standard development commands:

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
| `crates/application/` | Lint orchestration, output formatting, and exit-code policy |
| `crates/domain/` | Markdown model and lint rules |

## Security

Please report vulnerabilities as described in [SECURITY.md](SECURITY.md).

## License

Licensed under either the [Apache License 2.0](LICENSE-APACHE) or the [MIT License](LICENSE-MIT), at your
option.
