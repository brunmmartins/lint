/// A parsed Markdown document, as the domain rules see it: 1-indexed lines with their line
/// terminators stripped, plus the count of consecutive trailing newline characters at end-of-file.
///
/// Constructed only through [`Document::from_source`], which is infallible: any `&str` is a valid
/// source to line-split, so there is no invariant to reject at construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    lines: Vec<String>,
    trailing_newline_count: usize,
    source_is_empty: bool,
}

impl Document {
    /// Splits `source` into 1-indexed lines (the line terminator stripped from each), and records
    /// how many newline characters end the source consecutively: `0` means no trailing newline at
    /// all, `1` means exactly one, `2` or more means more than one.
    ///
    /// A `\r\n` line terminator has its `\r` stripped along with the `\n`, so line text never
    /// carries a trailing carriage return.
    #[must_use]
    pub fn from_source(source: &str) -> Self {
        if source.is_empty() {
            return Self {
                lines: Vec::new(),
                trailing_newline_count: 0,
                source_is_empty: true,
            };
        }

        let trailing_newline_count = source.chars().rev().take_while(|&c| c == '\n').count();

        let mut raw_lines: Vec<&str> = source.split('\n').collect();
        // `str::split('\n')` always yields one trailing empty element when `source` ends with
        // `\n` (that element is not a real line, only a marker that the newline was present); the
        // separately computed `trailing_newline_count` already carries the newline-count
        // information, so drop exactly that one phantom element here.
        if trailing_newline_count > 0 {
            raw_lines.pop();
        }

        let lines = raw_lines
            .into_iter()
            .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
            .collect();

        Self {
            lines,
            trailing_newline_count,
            source_is_empty: false,
        }
    }

    /// The document's lines, in order, 1-indexed by position (index `0` is line `1`).
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// How many newline characters end the source consecutively: `0` (no trailing newline), `1`
    /// (exactly one), or `2` or more (more than one).
    #[must_use]
    pub fn trailing_newline_count(&self) -> usize {
        self.trailing_newline_count
    }

    /// Whether the source that built this document had zero bytes.
    #[must_use]
    pub fn source_is_empty(&self) -> bool {
        self.source_is_empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_trailing_newline_is_counted_as_zero() {
        let doc = Document::from_source("abc");
        assert_eq!(doc.lines(), ["abc"]);
        assert_eq!(doc.trailing_newline_count(), 0);
    }

    #[test]
    fn single_trailing_newline_is_counted_as_one() {
        let doc = Document::from_source("abc\n");
        assert_eq!(doc.lines(), ["abc"]);
        assert_eq!(doc.trailing_newline_count(), 1);
    }

    #[test]
    fn two_trailing_newlines_add_one_blank_line_and_count_two() {
        let doc = Document::from_source("abc\n\n");
        assert_eq!(doc.lines(), ["abc", ""]);
        assert_eq!(doc.trailing_newline_count(), 2);
    }

    #[test]
    fn three_trailing_newlines_add_two_blank_lines_and_count_three() {
        let doc = Document::from_source("abc\n\n\n");
        assert_eq!(doc.lines(), ["abc", "", ""]);
        assert_eq!(doc.trailing_newline_count(), 3);
    }

    #[test]
    fn empty_source_has_no_lines_and_is_marked_empty() {
        let doc = Document::from_source("");
        assert!(doc.lines().is_empty());
        assert_eq!(doc.trailing_newline_count(), 0);
        assert!(doc.source_is_empty());
    }

    #[test]
    fn multiple_lines_split_on_newline() {
        let doc = Document::from_source("one\ntwo\nthree\n");
        assert_eq!(doc.lines(), ["one", "two", "three"]);
        assert_eq!(doc.trailing_newline_count(), 1);
    }

    #[test]
    fn crlf_line_terminators_strip_the_carriage_return() {
        let doc = Document::from_source("one\r\ntwo\r\n");
        assert_eq!(doc.lines(), ["one", "two"]);
        assert_eq!(doc.trailing_newline_count(), 1);
    }
}
