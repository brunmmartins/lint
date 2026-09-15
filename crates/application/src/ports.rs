use std::path::{Path, PathBuf};

use crate::{ReadFault, WalkFault};

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

/// Reads and UTF-8-validates one file's bytes, enforcing the size bound before allocating.
pub trait SourceReader {
    /// Reads `path`'s contents.
    ///
    /// # Errors
    ///
    /// Returns [`ReadFault::Unreadable`], [`ReadFault::TooLarge`], or [`ReadFault::InvalidUtf8`].
    fn read(&self, path: &Path) -> Result<String, ReadFault>;
}

/// Parses Markdown source into a [`lint_domain::Document`].
pub trait MarkdownParser {
    /// Parses `source` into its lines and the blocks the rules need, such as the line extents of
    /// fenced and indented code blocks. Infallible: every source is some Markdown document, and
    /// `pulldown-cmark`, the parser behind this port, does not error on malformed Markdown.
    fn parse(&self, source: &str) -> lint_domain::Document;
}
