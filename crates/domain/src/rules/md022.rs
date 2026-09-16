use super::line_text;
use crate::{Document, Finding, FindingMessage, Location, Rule, RuleId};

/// MD022 (`blanks-around-headings`): a heading has no blank line directly above it or directly
/// below it.
///
/// One blank line is required on each side. Extra blank lines are allowed, and a heading at the
/// very start or the very end of a file needs none beyond the file's edge. For a heading
/// underlined with `=` or `-`, the line below is the one after the underline. Both findings are
/// reported at column 1 of the heading's first line, the one above before the one below.
///
/// A line counts as blank when it is empty, when it holds only whitespace, and when nothing is
/// left of it once HTML comment text and block-quote markers are taken out. Front matter is not
/// recognised, so its lines are ordinary lines.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md022;

impl Rule for Md022 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        let lines = doc.lines();
        let last_line = lines.len();
        let mut findings = Vec::new();

        for block in doc.blocks() {
            if block.kind().heading_level().is_none() {
                continue;
            }
            let span = block.lines();
            let (first, last) = (span.first().get(), span.last().get());
            // The first line of a block is a 1-based line number and the column is 1, so
            // `Location::new` cannot fail; a failure would be a counting bug here.
            let Ok(location) = Location::new(first, 1) else {
                continue;
            };

            // A line outside the document counts as blank, so only a line that exists is tested.
            if first > 1 && !blank_at(lines, first - 1) {
                findings.push(Finding::new(
                    RuleId::Md022,
                    location,
                    FindingMessage::MissingBlankLineAboveHeading,
                ));
            }
            if last < last_line && !blank_at(lines, last + 1) {
                findings.push(Finding::new(
                    RuleId::Md022,
                    location,
                    FindingMessage::MissingBlankLineBelowHeading,
                ));
            }
        }
        findings
    }
}

/// Whether the 1-based line `number` of `lines` is blank. A line beyond the document is blank.
fn blank_at(lines: &[String], number: usize) -> bool {
    lines
        .get(number.saturating_sub(1))
        .is_none_or(|line| line_text::is_blank_line(line))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlockKind, HeadingLevel};

    /// Whether the finding is the "above" one.
    fn is_above(message: FindingMessage) -> bool {
        message == FindingMessage::MissingBlankLineAboveHeading
    }

    /// `(line, "above" or "below")` for every finding, in report order.
    fn reported(doc: &Document) -> Vec<(usize, &'static str)> {
        Md022
            .check(doc)
            .iter()
            .map(|finding| {
                assert_eq!(finding.rule(), RuleId::Md022);
                assert_eq!(finding.location().col().get(), 1);
                let side = if is_above(finding.message()) {
                    "above"
                } else {
                    assert_eq!(
                        finding.message(),
                        FindingMessage::MissingBlankLineBelowHeading
                    );
                    "below"
                };
                (finding.location().line().get(), side)
            })
            .collect()
    }

    /// A document whose headings span the given 1-based inclusive line ranges. The level does not
    /// matter to this rule, so every heading is a level-1 one.
    fn document(source: &str, headings: &[(usize, usize)]) -> Document {
        let blocks: Vec<_> = headings
            .iter()
            .map(|&(first, last)| {
                (
                    BlockKind::Heading(HeadingLevel::H1),
                    line_range(source, first, last),
                )
            })
            .collect();
        Document::from_source_and_blocks(source, blocks)
    }

    /// The byte range covering lines `first` through `last` (1-based), without the terminator of
    /// the last line.
    fn line_range(source: &str, first: usize, last: usize) -> std::ops::Range<usize> {
        let mut starts = vec![0];
        starts.extend(
            source
                .bytes()
                .enumerate()
                .filter(|&(_, byte)| byte == b'\n')
                .map(|(offset, _)| offset + 1),
        );
        let start = starts[first - 1];
        let end = starts.get(last).map_or(source.len(), |&next| next - 1);
        start..end.max(start)
    }

    #[test]
    fn reports_above_then_below_on_the_first_line() {
        let doc = document("Intro\n# Heading\nText\n", &[(2, 2)]);
        assert_eq!(reported(&doc), [(2, "above"), (2, "below")]);
    }

    #[test]
    fn lines_beyond_the_file_edges_count_as_blank() {
        let doc = document("# Top\ntext\n\ntext\n## Two\n", &[(1, 1), (5, 5)]);
        assert_eq!(reported(&doc), [(1, "below"), (5, "above")]);

        // A heading on the last line, with no final newline.
        let doc = document("text\n\n# Last", &[(3, 3)]);
        assert!(reported(&doc).is_empty());
    }

    #[test]
    fn extra_blank_lines_are_allowed() {
        let doc = document("text\n\n\n# Heading\n\n\ntext\n", &[(4, 4)]);
        assert!(reported(&doc).is_empty());
    }

    #[test]
    fn setext_heading_is_checked_below_its_underline() {
        // The heading spans its content line and its underline; the line below the underline is
        // the one tested, and the finding lands on the first line.
        let doc = document("para\n\nTitle\n===\nbody\n", &[(3, 4)]);
        assert_eq!(reported(&doc), [(3, "below")]);

        let doc = document("para\n\nline a\nline b\n===\nnext\n", &[(3, 5)]);
        assert_eq!(reported(&doc), [(3, "below")]);

        let doc = document(
            "# First\n\nSetext\n------\n\n## Last\n",
            &[(1, 1), (3, 4), (6, 6)],
        );
        assert!(reported(&doc).is_empty());
    }

    #[test]
    fn quote_markers_and_comments_count_as_blank() {
        let source = "> text\n> # Quoted\n>\n> more\n\n<!-- note -->\n# After comment\n";
        let doc = document(source, &[(2, 2), (7, 7)]);
        assert_eq!(reported(&doc), [(2, "above")]);
    }

    #[test]
    fn text_before_an_unclosed_comment_is_not_blank() {
        let source = "# A\n  \n## B\n>\n### C\n<!-- x -->\n#### D\nx <!-- y\n##### E\n";
        let doc = document(source, &[(1, 1), (3, 3), (5, 5), (7, 7), (9, 9)]);
        assert_eq!(reported(&doc), [(7, "below"), (9, "above")]);
    }

    #[test]
    fn a_document_without_headings_is_clean() {
        assert!(reported(&Document::from_source("text\ntext\n")).is_empty());
    }
}
