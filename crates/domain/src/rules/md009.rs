use crate::{Document, Finding, FindingMessage, Location, Rule, RuleId};

/// MD009 (`no-trailing-spaces`): a line ends with one or more trailing space or tab characters.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md009;

impl Rule for Md009 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (index, line) in doc.lines().iter().enumerate() {
            let trimmed = line.trim_end_matches([' ', '\t']);
            if trimmed.chars().count() == line.chars().count() {
                continue;
            }
            let line_number = index + 1;
            let column = trimmed.chars().count() + 1;
            // `line_number` and `column` are both derived from a 1-based count over an existing
            // line, so `Location::new` cannot fail here; a failure would mean this rule's own
            // counting is broken, not that the input was invalid.
            let Ok(location) = Location::new(line_number, column) else {
                continue;
            };
            findings.push(Finding::new(
                RuleId::Md009,
                location,
                FindingMessage::TrailingWhitespace,
            ));
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_line_reports_nothing() {
        let doc = Document::from_source("no trailing space\n");
        assert!(Md009.check(&doc).is_empty());
    }

    #[test]
    fn trailing_spaces_are_reported_at_the_first_trailing_column() {
        let doc = Document::from_source("abc  \n");
        let findings = Md009.check(&doc);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule(), RuleId::Md009);
        assert_eq!(findings[0].location(), Location::new(1, 4).unwrap());
    }

    #[test]
    fn trailing_tab_is_reported() {
        let doc = Document::from_source("abc\t\n");
        let findings = Md009.check(&doc);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].location(), Location::new(1, 4).unwrap());
    }

    #[test]
    fn leading_and_internal_spaces_alone_report_nothing() {
        let doc = Document::from_source("  abc def\n");
        assert!(Md009.check(&doc).is_empty());
    }

    #[test]
    fn each_offending_line_is_reported_independently() {
        let doc = Document::from_source("clean\ntrail  \nclean again\ntrail again \n");
        let findings = Md009.check(&doc);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].location(), Location::new(2, 6).unwrap());
        assert_eq!(findings[1].location(), Location::new(4, 12).unwrap());
    }
}
