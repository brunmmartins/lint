use crate::{Document, Finding, FindingMessage, Location, Rule, RuleId};

/// MD010 (`no-hard-tabs`): a line contains a hard tab character.
///
/// Every line is checked, including lines in code blocks and code spans. Each run of consecutive
/// tabs is one finding, at the column of the run's first tab. Columns count characters, and a tab
/// counts as one character.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md010;

impl Rule for Md010 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        let mut findings = Vec::new();
        for (index, line) in doc.lines().iter().enumerate() {
            if !line.contains('\t') {
                continue;
            }
            let line_number = index + 1;
            let mut previous_is_tab = false;
            for (char_index, character) in line.chars().enumerate() {
                let is_tab = character == '\t';
                if is_tab && !previous_is_tab {
                    // Both numbers are 1-based counts over an existing line, so `Location::new`
                    // cannot fail; a failure would be a counting bug here, not invalid input.
                    if let Ok(location) = Location::new(line_number, char_index + 1) {
                        findings.push(Finding::new(
                            RuleId::Md010,
                            location,
                            FindingMessage::HardTab,
                        ));
                    }
                }
                previous_is_tab = is_tab;
            }
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locations(source: &str) -> Vec<(usize, usize)> {
        Md010
            .check(&Document::from_source(source))
            .iter()
            .map(|finding| {
                assert_eq!(finding.rule(), RuleId::Md010);
                assert_eq!(finding.message(), FindingMessage::HardTab);
                let location = finding.location();
                (location.line().get(), location.col().get())
            })
            .collect()
    }

    #[test]
    fn one_finding_per_run_of_tabs() {
        assert_eq!(
            locations("\tindented\na\t\tb\tc\n"),
            [(1, 1), (2, 2), (2, 5)]
        );
    }

    #[test]
    fn spaces_only_is_clean() {
        assert!(locations("    indented\n  - item\n").is_empty());
    }

    #[test]
    fn tabs_at_the_end_of_a_line_are_reported() {
        assert_eq!(locations("abc\t\t\n"), [(1, 4)]);
    }

    #[test]
    fn column_counts_characters() {
        assert_eq!(locations("é\tx\n"), [(1, 2)]);
        assert_eq!(locations("日本\t語\n"), [(1, 3)]);
    }
}
