use std::path::{Path, PathBuf};

use crate::{FileFindings, LintFault, ReadFault, ReportFault, WalkFault};

/// Resolves one CLI input (a file or a directory) into zero or more Markdown file paths to lint.
pub trait Walker {
    /// Resolves `input`. For a directory, returns the Markdown files found under it, in
    /// sorted-by-name, depth-first order; for a file, returns that one path.
    ///
    /// # Errors
    ///
    /// Returns [`WalkFault::NotFound`] when `input` does not exist, or
    /// [`WalkFault::Unreadable`] when a directory entry under it could not be listed.
    fn resolve(&self, input: &Path) -> Result<Vec<PathBuf>, WalkFault>;
}

/// Reads and UTF-8-validates one regular file's bytes, within a fixed size bound.
///
/// An implementation rejects any path that is not a regular file, or a symlink to one, without
/// reading from it, so a named pipe or a device can neither block the run nor feed it without end.
/// It never reads more than one byte past the size bound, even from a file that grows while it is
/// being read.
pub trait SourceReader {
    /// Reads `path`'s contents.
    ///
    /// # Errors
    ///
    /// Returns [`ReadFault::NotRegularFile`] when `path` is not a regular file,
    /// [`ReadFault::TooLarge`] when it holds more bytes than the size bound,
    /// [`ReadFault::InvalidUtf8`] when its bytes are not UTF-8, or [`ReadFault::Unreadable`] for
    /// any other failure.
    fn read(&self, path: &Path) -> Result<String, ReadFault>;
}

/// Parses Markdown source into a [`lint_domain::Document`].
pub trait MarkdownParser {
    /// Parses `source` into its lines and the blocks the rules need, such as the line extents of
    /// fenced and indented code blocks. Infallible: every source is some Markdown document, and
    /// `pulldown-cmark`, the parser behind this port, does not error on malformed Markdown.
    fn parse(&self, source: &str) -> lint_domain::Document;
}

/// Delivers results as they are produced: each file's findings, each fault, and the end of the run.
///
/// [`run_lint`](crate::run_lint) calls it file by file, so no file's results are held until the
/// run ends. After [`report_findings`](Reporter::report_findings) or
/// [`finish`](Reporter::finish) returns an error, the run stops and the reporter receives no
/// further call, except one [`report_fault`](Reporter::report_fault) with
/// [`LintFault::OutputUnwritable`] when the error was [`ReportFault::Unwritable`].
pub trait Reporter {
    /// Writes every finding of one file, in the order given, and returns once they have been
    /// delivered. An implementation may buffer within this call, but must not hold findings past
    /// its return.
    ///
    /// # Errors
    ///
    /// Returns [`ReportFault::Closed`] when the output's reader has gone away, or
    /// [`ReportFault::Unwritable`] for any other write or flush failure.
    fn report_findings(&mut self, file: &FileFindings) -> Result<(), ReportFault>;

    /// Reports one fault as it occurs. Delivery is best effort: a fault that cannot be written is
    /// dropped, because the run's exit status already reports it.
    fn report_fault(&mut self, fault: &LintFault);

    /// Completes the output after the last input, delivering anything still buffered.
    ///
    /// # Errors
    ///
    /// Returns [`ReportFault::Closed`] or [`ReportFault::Unwritable`], as for
    /// [`report_findings`](Reporter::report_findings).
    fn finish(&mut self) -> Result<(), ReportFault>;
}
