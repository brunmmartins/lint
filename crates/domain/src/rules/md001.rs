use crate::{Document, Finding, FindingMessage, HeadingLevel, Location, Rule, RuleId};

/// MD001 (`heading-increment`): a heading is more than one level deeper than the heading before
/// it, so a level is skipped.
///
/// Headings are read in document order, including those written with `#` characters, those
/// underlined with `=` or `-`, and those nested in lists and block quotes. The first heading of a
/// document may have any level, and moving back up any number of levels is allowed. A heading that
/// is reported still sets the level the next heading is compared against. The finding is reported
/// at column 1 of the heading's first line.
///
/// Front matter is not recognised, so a `title` in it does not count as a heading.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md001;

impl Rule for Md001 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut previous: Option<HeadingLevel> = None;

        for block in doc.blocks() {
            let Some(level) = block.kind().heading_level() else {
                continue;
            };

            // `deeper` is `None` only for the deepest level, which no level can be deeper than, so
            // a skip always has an expected level.
            let expected = previous
                .filter(|before| level.number() > before.number().saturating_add(1))
                .and_then(HeadingLevel::deeper);
            if let Some(expected) = expected {
                // The first line of a block is a 1-based line number and the column is 1, so
                // `Location::new` cannot fail; a failure would be a counting bug here.
                if let Ok(location) = Location::new(block.lines().first().get(), 1) {
                    findings.push(Finding::new(
                        RuleId::Md001,
                        location,
                        FindingMessage::HeadingLevelSkipped {
                            expected,
                            found: level,
                        },
                    ));
                }
            }

            previous = Some(level);
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;

    /// Builds a document of `count` one-line blocks, block `n` spanning line `n + 1`.
    fn document(kinds: &[BlockKind]) -> Document {
        let source = "x\n".repeat(kinds.len());
        let blocks: Vec<_> = kinds
            .iter()
            .enumerate()
            .map(|(index, &kind)| (kind, index * 2..index * 2 + 1))
            .collect();
        Document::from_source_and_blocks(&source, blocks)
    }

    fn heading(level: HeadingLevel) -> BlockKind {
        BlockKind::Heading(level)
    }

    /// `(line, expected, found)` for every finding.
    fn reported(doc: &Document) -> Vec<(usize, u8, u8)> {
        Md001
            .check(doc)
            .iter()
            .map(|finding| {
                assert_eq!(finding.rule(), RuleId::Md001);
                assert_eq!(finding.location().col().get(), 1);
                let FindingMessage::HeadingLevelSkipped { expected, found } = finding.message()
                else {
                    panic!("unexpected message {:?}", finding.message());
                };
                (
                    finding.location().line().get(),
                    expected.number(),
                    found.number(),
                )
            })
            .collect()
    }

    #[test]
    fn reports_each_skip_with_expected_and_found() {
        let doc = document(&[
            heading(HeadingLevel::H1),
            heading(HeadingLevel::H3),
            heading(HeadingLevel::H4),
            heading(HeadingLevel::H6),
        ]);
        assert_eq!(reported(&doc), [(2, 2, 3), (4, 5, 6)]);
    }

    #[test]
    fn a_reported_heading_sets_the_next_expected_level() {
        // H1 then H3 is reported; H4 that follows is one deeper than H3, so it is not.
        let doc = document(&[
            heading(HeadingLevel::H1),
            heading(HeadingLevel::H3),
            heading(HeadingLevel::H4),
        ]);
        assert_eq!(reported(&doc), [(2, 2, 3)]);
    }

    #[test]
    fn moving_up_any_number_of_levels_is_allowed() {
        let doc = document(&[
            heading(HeadingLevel::H1),
            heading(HeadingLevel::H2),
            heading(HeadingLevel::H6),
            heading(HeadingLevel::H1),
            heading(HeadingLevel::H2),
        ]);
        assert_eq!(reported(&doc), [(3, 3, 6)]);
    }

    #[test]
    fn the_first_heading_may_have_any_level() {
        for level in [HeadingLevel::H1, HeadingLevel::H4, HeadingLevel::H6] {
            let doc = document(&[heading(level)]);
            assert!(reported(&doc).is_empty(), "{level:?}");
        }
    }

    #[test]
    fn blocks_that_are_not_headings_are_ignored() {
        let doc = document(&[
            heading(HeadingLevel::H1),
            BlockKind::BlockQuote,
            BlockKind::Html,
            BlockKind::FencedCode,
            BlockKind::List,
            heading(HeadingLevel::H3),
        ]);
        assert_eq!(reported(&doc), [(6, 2, 3)]);
    }

    #[test]
    fn a_document_without_headings_is_clean() {
        assert!(reported(&Document::from_source("text\n")).is_empty());
        assert!(reported(&Document::from_source("")).is_empty());
    }
}
