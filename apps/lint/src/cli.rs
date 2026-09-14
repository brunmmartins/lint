use std::path::PathBuf;

use clap::Parser;

/// Lint Markdown files for style problems.
///
/// Exit status: `0` when no finding remains, `1` when at least one finding remains, `2` on a
/// usage, configuration, or I/O error (brief §6).
#[derive(Debug, Parser)]
#[command(name = "lint", version, about)]
pub struct Cli {
    /// One or more file or directory paths to lint. A directory is walked recursively.
    #[arg(required = true, num_args = 1..)]
    pub paths: Vec<PathBuf>,
}
