use std::fmt;
use std::num::NonZeroUsize;

/// A 1-based line and column position inside a [`Document`](crate::Document).
///
/// Both `line` and `col` are `NonZeroUsize`: the `path:line:col` output format is
/// 1-based, and [`Location::new`] is the only place a zero value is rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    line: NonZeroUsize,
    col: NonZeroUsize,
}

/// The error returned by [`Location::new`] when a line or column of `0` is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationError {
    /// `line` was `0`; lines are 1-based.
    ZeroLine,
    /// `col` was `0`; columns are 1-based.
    ZeroCol,
}

impl fmt::Display for LocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationError::ZeroLine => write!(f, "line must be 1 or greater"),
            LocationError::ZeroCol => write!(f, "column must be 1 or greater"),
        }
    }
}

impl std::error::Error for LocationError {}

impl Location {
    /// Builds a [`Location`], rejecting a `line` or `col` of `0`.
    ///
    /// # Errors
    ///
    /// Returns [`LocationError::ZeroLine`] or [`LocationError::ZeroCol`] when the corresponding
    /// argument is `0`.
    pub fn new(line: usize, col: usize) -> Result<Self, LocationError> {
        let line = NonZeroUsize::new(line).ok_or(LocationError::ZeroLine)?;
        let col = NonZeroUsize::new(col).ok_or(LocationError::ZeroCol)?;
        Ok(Self { line, col })
    }

    /// The 1-based line number.
    #[must_use]
    pub fn line(&self) -> NonZeroUsize {
        self.line
    }

    /// The 1-based column number.
    #[must_use]
    pub fn col(&self) -> NonZeroUsize {
        self.col
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_positive_line_and_col() {
        let location = Location::new(3, 5).unwrap();
        assert_eq!(location.line().get(), 3);
        assert_eq!(location.col().get(), 5);
    }

    #[test]
    fn new_rejects_zero_line() {
        assert_eq!(Location::new(0, 1), Err(LocationError::ZeroLine));
    }

    #[test]
    fn new_rejects_zero_col() {
        assert_eq!(Location::new(1, 0), Err(LocationError::ZeroCol));
    }

    #[test]
    fn display_formats_as_line_colon_col() {
        let location = Location::new(3, 5).unwrap();
        assert_eq!(location.to_string(), "3:5");
    }
}
