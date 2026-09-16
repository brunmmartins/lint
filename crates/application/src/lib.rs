//! `lint-application`: the [`Walker`], [`SourceReader`], [`MarkdownParser`], and [`Reporter`]
//! ports, the [`LintFault`] taxonomy, the [`run_lint`] use case, and the pure [`format_finding`]
//! and [`decide_exit_code`] functions.
//!
//! [`run_lint`] streams: each file's findings and each fault go to the [`Reporter`] as they are
//! produced, and a failing output stops the run. Only regular files are read, within a size
//! bound.
//!
//! Depends on `lint-domain` only: no `pulldown-cmark`, `clap`, `std::fs`, or `std::env` here
//! either — those live behind the ports, implemented by adapters in `apps/lint`.

mod fault;
mod format;
mod ports;
mod run_lint;

pub use fault::{LintFault, ReadFault, ReportFault, WalkFault};
pub use format::{decide_exit_code, format_finding};
pub use ports::{MarkdownParser, Reporter, SourceReader, Walker};
pub use run_lint::{FileFindings, LintOutcome, run_lint};
