//! `lint`'s composition root: the only place that parses arguments, touches the file system, or
//! calls `pulldown-cmark`.
//!
//! Findings and faults are written as each file is checked, through one reporter that owns
//! standard output and standard error. When standard output closes or cannot be written, the run
//! stops early and exits with status 2.

mod cli;
mod parser;
mod reader;
mod reporter;
mod walker;

use std::io::{self, BufWriter};
use std::process::ExitCode;

use clap::Parser as _;
use lint_application::run_lint;

use cli::Cli;
use parser::PulldownMarkdownParser;
use reader::StdSourceReader;
use reporter::TextReporter;
use walker::StdWalker;

fn main() -> ExitCode {
    // A missing or invalid argument exits through clap's own usage-error path: a usage message on
    // stderr and exit code 2, with no custom error handling.
    let cli = Cli::parse();

    // Standard output is buffered and flushed after each file's findings; standard error is
    // unbuffered, so a fault appears as soon as it is reported.
    let mut reporter = TextReporter::new(BufWriter::new(io::stdout().lock()), io::stderr().lock());

    let outcome = run_lint(
        &cli.paths,
        &StdWalker,
        &StdSourceReader,
        &PulldownMarkdownParser,
        &mut reporter,
    );

    // After a stop, dropping the buffered writer may retry a write that fails again; that error is
    // discarded, and the stop is already reflected in the outcome.
    drop(reporter);
    ExitCode::from(outcome.exit_code())
}
