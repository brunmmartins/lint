//! `lint-application`: the [`Walker`], [`SourceReader`], and [`MarkdownParser`] ports, the
//! [`LintFault`] taxonomy, the [`run_lint`] use case, and the pure [`format_finding`] and
//! [`decide_exit_code`] functions.
//!
//! Depends on `lint-domain` only: no `pulldown-cmark`, `clap`, `std::fs`, or `std::env` here
//! either — those live behind the ports, implemented by adapters in `apps/lint`.

mod fault;
mod format;
mod ports;
mod run_lint;

pub use fault::{LintFault, ReadFault, WalkFault};
pub use format::{decide_exit_code, format_finding};
pub use ports::{MarkdownParser, SourceReader, Walker};
pub use run_lint::{FileFindings, LintOutcome, run_lint};
