use crate::{Document, Finding, FindingMessage, Location, Rule, RuleId};

/// MD047 (`single-trailing-newline`): a file must end with exactly one trailing newline.
///
/// A completely empty file (zero bytes) is not flagged: there is no content for a trailing
/// newline to apply to.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md047;

impl Rule for Md047 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        if doc.source_is_empty() {
            return Vec::new();
        }

        match doc.trailing_newline_count() {
            1 => Vec::new(),
            0 => {
                let last_line_number = doc.lines().len().max(1);
                // `last_line_number` is always >= 1 here because a non-empty source produces at
                // least one line, so `Location::new` cannot fail; treat failure as "nothing to
                // report" rather than panicking on an internal accounting bug.
                let last_line = doc.lines().last().map(String::as_str).unwrap_or_default();
                let column = last_line.chars().count() + 1;
                Location::new(last_line_number, column).map_or_else(
                    |_| Vec::new(),
                    |location| {
                        vec![Finding::new(
                            RuleId::Md047,
                            location,
                            FindingMessage::MissingTrailingNewline,
                        )]
                    },
                )
            }
            _ => {
                let last_line_number = doc.lines().len().max(1);
                Location::new(last_line_number, 1).map_or_else(
                    |_| Vec::new(),
                    |location| {
                        vec![Finding::new(
                            RuleId::Md047,
                            location,
                            FindingMessage::MultipleTrailingNewlines,
                        )]
                    },
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exactly_one_trailing_newline_reports_nothing() {
        let doc = Document::from_source("abc\n");
        assert!(Md047.check(&doc).is_empty());
    }

    #[test]
    fn empty_file_reports_nothing() {
        let doc = Document::from_source("");
        assert!(Md047.check(&doc).is_empty());
    }

    #[test]
    fn missing_trailing_newline_is_reported_at_end_of_last_line() {
        let doc = Document::from_source("abc");
        let findings = Md047.check(&doc);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule(), RuleId::Md047);
        assert_eq!(findings[0].location(), Location::new(1, 4).unwrap());
    }

    #[test]
    fn multiple_trailing_newlines_are_reported() {
        let doc = Document::from_source("abc\n\n\n");
        let findings = Md047.check(&doc);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location(), Location::new(3, 1).unwrap());
    }
}
