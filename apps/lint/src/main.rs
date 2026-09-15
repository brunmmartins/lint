//! `lint`'s composition root: the only place that parses arguments, touches the file system, or
//! calls `pulldown-cmark`.

mod cli;
mod parser;
mod reader;
mod walker;

use clap::Parser as _;
use lint_application::{decide_exit_code, format_finding, run_lint};

use cli::Cli;
use parser::PulldownMarkdownParser;
use reader::StdSourceReader;
use walker::StdWalker;

fn main() -> std::process::ExitCode {
    // A missing or invalid argument exits through clap's own usage-error path: a usage message on
    // stderr and exit code 2, with no custom error handling.
    let cli = Cli::parse();

    let walker = StdWalker;
    let reader = StdSourceReader;
    let parser = PulldownMarkdownParser;

    let outcome = run_lint(&cli.paths, &walker, &reader, &parser);

    for (path, finding) in &outcome.findings {
        println!("{}", format_finding(path, finding));
    }
    for fault in &outcome.faults {
        eprintln!("{fault}");
    }

    let code = decide_exit_code(!outcome.faults.is_empty(), !outcome.findings.is_empty());
    std::process::ExitCode::from(code)
}
