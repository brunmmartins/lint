use std::fmt;

/// The message of a [`Finding`](crate::Finding), rendered through `Display`.
///
/// The set of messages is closed and no variant carries text, so a message can never echo the
/// contents of the file being checked. Numbers in a message are limits and counts the rule
/// defines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FindingMessage {
    /// Renders as "trailing whitespace".
    TrailingWhitespace,
    /// Renders as "missing single trailing newline".
    MissingTrailingNewline,
    /// Renders as "more than one trailing newline".
    MultipleTrailingNewlines,
    /// Renders as "hard tab".
    HardTab,
    /// Renders as "multiple consecutive blank lines (expected at most {maximum}, found {found})".
    MultipleBlankLines {
        /// The most consecutive blank lines allowed.
        maximum: usize,
        /// How many consecutive blank lines there are, up to and including the reported one.
        found: usize,
    },
    /// Renders as "line too long (expected at most {maximum} characters, found {found})".
    LineTooLong {
        /// The most characters a line may have.
        maximum: usize,
        /// How many characters the line has.
        found: usize,
    },
}

impl fmt::Display for FindingMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FindingMessage::TrailingWhitespace => f.write_str("trailing whitespace"),
            FindingMessage::MissingTrailingNewline => {
                f.write_str("missing single trailing newline")
            }
            FindingMessage::MultipleTrailingNewlines => {
                f.write_str("more than one trailing newline")
            }
            FindingMessage::HardTab => f.write_str("hard tab"),
            FindingMessage::MultipleBlankLines { maximum, found } => write!(
                f,
                "multiple consecutive blank lines (expected at most {maximum}, found {found})"
            ),
            FindingMessage::LineTooLong { maximum, found } => write!(
                f,
                "line too long (expected at most {maximum} characters, found {found})"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_message_renders_its_exact_text() {
        let cases = [
            (FindingMessage::TrailingWhitespace, "trailing whitespace"),
            (
                FindingMessage::MissingTrailingNewline,
                "missing single trailing newline",
            ),
            (
                FindingMessage::MultipleTrailingNewlines,
                "more than one trailing newline",
            ),
            (FindingMessage::HardTab, "hard tab"),
            (
                FindingMessage::MultipleBlankLines {
                    maximum: 1,
                    found: 3,
                },
                "multiple consecutive blank lines (expected at most 1, found 3)",
            ),
            (
                FindingMessage::LineTooLong {
                    maximum: 80,
                    found: 95,
                },
                "line too long (expected at most 80 characters, found 95)",
            ),
        ];

        for (message, text) in cases {
            assert_eq!(message.to_string(), text);
        }
    }
}
