use crate::{Document, Finding, FindingMessage, Location, Rule, RuleId};

/// MD013 (`line-length`): a line is longer than 80 characters and could be wrapped.
///
/// A long line is reported at column 81 only when it has whitespace at column 81 or later. A line
/// whose overflow holds no whitespace, such as a long URL, cannot be wrapped there and is not
/// reported. Headings, code blocks, and tables share the same limit. Lengths count characters,
/// and a tab counts as one character.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md013;

/// The most characters a line may have.
const LINE_LENGTH: usize = 80;

impl Rule for Md013 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (index, line) in doc.lines().iter().enumerate() {
            // A line of at most `LINE_LENGTH` bytes has at most that many characters.
            if line.len() <= LINE_LENGTH {
                continue;
            }

            let mut length = 0_usize;
            let mut whitespace_beyond_limit = false;
            for character in line.chars() {
                length = length.saturating_add(1);
                if length > LINE_LENGTH && character.is_whitespace() {
                    whitespace_beyond_limit = true;
                }
            }
            if length <= LINE_LENGTH || !whitespace_beyond_limit {
                continue;
            }

            // The line number is a 1-based count over an existing line and the column is a
            // positive constant, so `Location::new` cannot fail; a failure would be a counting bug.
            if let Ok(location) = Location::new(index + 1, LINE_LENGTH + 1) {
                findings.push(Finding::new(
                    RuleId::Md013,
                    location,
                    FindingMessage::LineTooLong {
                        maximum: LINE_LENGTH,
                        found: length,
                    },
                ));
            }
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `(line, column, found)` for every finding.
    fn reported(source: &str) -> Vec<(usize, usize, usize)> {
        Md013
            .check(&Document::from_source(source))
            .iter()
            .map(|finding| {
                assert_eq!(finding.rule(), RuleId::Md013);
                let FindingMessage::LineTooLong { maximum, found } = finding.message() else {
                    panic!("unexpected message {:?}", finding.message());
                };
                assert_eq!(maximum, 80);
                let location = finding.location();
                (location.line().get(), location.col().get(), found)
            })
            .collect()
    }

    #[test]
    fn eighty_characters_is_clean() {
        let line = "word ".repeat(15) + "abcde";
        assert_eq!(line.chars().count(), 80);
        assert!(reported(&(line + "\n")).is_empty());
    }

    #[test]
    fn reports_whitespace_beyond_limit() {
        let source = format!("{}\n{}\n", "a".repeat(89) + " bbbbb", "a".repeat(80) + " b");
        assert_eq!(reported(&source), [(1, 81, 95), (2, 81, 82)]);
    }

    #[test]
    fn no_whitespace_beyond_limit_is_exempt() {
        let source = format!(
            "{}\n{}\n{}\n",
            "x".repeat(120),
            "a".repeat(79) + " https://example.com/" + &"x".repeat(30),
            "a ".repeat(30) + &"b".repeat(70),
        );
        assert!(reported(&source).is_empty());
    }

    #[test]
    fn tab_beyond_limit_is_whitespace() {
        let source = "a".repeat(85) + "\tb\n";
        assert_eq!(reported(&source), [(1, 81, 87)]);
    }

    #[test]
    fn counts_characters_not_bytes() {
        let eighty = "é".repeat(40) + " " + &"é".repeat(39);
        assert_eq!(eighty.len(), 159);
        let source = format!(
            "{eighty}\n{}\n{}\n",
            "é".repeat(81) + " é",
            "\t".to_string() + &"a".repeat(79),
        );
        assert_eq!(reported(&source), [(2, 81, 83)]);
    }
}
