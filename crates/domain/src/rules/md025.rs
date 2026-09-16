use super::line_text;
use crate::{Block, Document, Finding, FindingMessage, HeadingLevel, Location, Rule, RuleId};

/// MD025 (`single-h1`): a document whose first content is a level-1 heading has another level-1
/// heading later on.
///
/// The first level-1 heading is the document's title only when it is not inside a list or a block
/// quote, and everything before it is blank lines or HTML comments. Without such a title, nothing
/// is reported. With one, every later level-1 heading is reported at column 1 of its first line,
/// whether it is written with `#`, underlined with `=`, or nested in a list or a block quote.
///
/// Front matter is not recognised, so a `title` in it is neither a heading nor allowed before one.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md025;

impl Rule for Md025 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        let blocks = doc.blocks();
        let Some(title_index) = blocks.iter().position(is_top_level_heading) else {
            return Vec::new();
        };
        let Some(title) = blocks.get(title_index) else {
            return Vec::new();
        };
        let before = blocks.get(..title_index).unwrap_or_default();

        if !is_title(doc, before, title) {
            return Vec::new();
        }

        let mut findings = Vec::new();
        for block in blocks.iter().skip(title_index.saturating_add(1)) {
            if !is_top_level_heading(block) {
                continue;
            }
            // The first line of a block is a 1-based line number and the column is 1, so
            // `Location::new` cannot fail; a failure would be a counting bug here.
            if let Ok(location) = Location::new(block.lines().first().get(), 1) {
                findings.push(Finding::new(
                    RuleId::Md025,
                    location,
                    FindingMessage::MultipleTopLevelHeadings,
                ));
            }
        }
        findings
    }
}

/// Whether `block` is a level-1 heading.
fn is_top_level_heading(block: &Block) -> bool {
    block.kind().heading_level() == Some(HeadingLevel::H1)
}

/// Whether the level-1 heading `title`, which `before` precedes in `blocks()` order, is the
/// document's title.
fn is_title(doc: &Document, before: &[Block], title: &Block) -> bool {
    let first_line = title.lines().first().get();

    // A container that reaches the heading's first line holds it, so the heading is nested. The
    // blocks are ordered by first line and a container is supplied before what it holds, so this
    // one pass sees every container that could hold the heading.
    if before
        .iter()
        .any(|block| block.kind().is_container() && block.lines().last().get() >= first_line)
    {
        return false;
    }

    only_blank_lines_and_comments_before(doc, before, first_line)
}

/// Whether every line before the 1-based `first_line` is whitespace, or belongs to an HTML block
/// holding nothing but a comment.
fn only_blank_lines_and_comments_before(
    doc: &Document,
    before: &[Block],
    first_line: usize,
) -> bool {
    let lines = doc.lines();
    // One pass over the lines and the blocks together, as `blocks()` is ordered by first line.
    let mut pending_blocks = before.iter().peekable();
    let mut commented_through = 0;

    for line_number in 1..first_line {
        while let Some(block) =
            pending_blocks.next_if(|block| block.lines().first().get() <= line_number)
        {
            if !block.kind().is_html() {
                continue;
            }
            let span = block.lines();
            let (first, last) = (span.first().get(), span.last().get());
            let block_lines = lines.get(first.saturating_sub(1)..last).unwrap_or_default();
            if line_text::is_comment_block(block_lines) {
                commented_through = commented_through.max(last);
            }
        }

        if line_number <= commented_through {
            continue;
        }
        let is_blank = lines
            .get(line_number.saturating_sub(1))
            .is_none_or(|line| line.trim().is_empty());
        if !is_blank {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;

    /// The line numbers reported for `doc`.
    fn reported(doc: &Document) -> Vec<usize> {
        Md025
            .check(doc)
            .iter()
            .map(|finding| {
                assert_eq!(finding.rule(), RuleId::Md025);
                assert_eq!(finding.location().col().get(), 1);
                assert_eq!(finding.message(), FindingMessage::MultipleTopLevelHeadings);
                finding.location().line().get()
            })
            .collect()
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

    /// A document with the given `(kind, first line, last line)` blocks, supplied in that order.
    fn document(source: &str, blocks: &[(BlockKind, usize, usize)]) -> Document {
        let mapped: Vec<_> = blocks
            .iter()
            .map(|&(kind, first, last)| (kind, line_range(source, first, last)))
            .collect();
        Document::from_source_and_blocks(source, mapped)
    }

    const H1: BlockKind = BlockKind::Heading(HeadingLevel::H1);
    const H2: BlockKind = BlockKind::Heading(HeadingLevel::H2);

    #[test]
    fn every_later_level_one_heading_is_reported() {
        let source = "# Title\n\n## Part\n\n# Second\n\nOther\n=====\n";
        let doc = document(source, &[(H1, 1, 1), (H2, 3, 3), (H1, 5, 5), (H1, 7, 8)]);
        assert_eq!(reported(&doc), [5, 7]);
    }

    #[test]
    fn nested_later_level_one_headings_are_reported() {
        let source = "# Title\n\n- # In list\n";
        let doc = document(source, &[(H1, 1, 1), (BlockKind::List, 3, 3), (H1, 3, 3)]);
        assert_eq!(reported(&doc), [3]);
    }

    #[test]
    fn blank_lines_and_comment_blocks_may_precede_the_title() {
        let source = "\n<!-- c -->\n  \n# Title\n\n# Two\n";
        let doc = document(source, &[(BlockKind::Html, 2, 2), (H1, 4, 4), (H1, 6, 6)]);
        assert_eq!(reported(&doc), [6]);
    }

    #[test]
    fn a_multi_line_comment_block_may_precede_the_title() {
        let source = "<!--\nc\n-->\n\n# Title\n\n# Two\n";
        let doc = document(source, &[(BlockKind::Html, 1, 3), (H1, 5, 5), (H1, 7, 7)]);
        assert_eq!(reported(&doc), [7]);
    }

    #[test]
    fn content_before_the_first_level_one_heading_means_no_title() {
        // A paragraph, which produces no block at all.
        let source = "Intro.\n\n# One\n\n# Two\n";
        let doc = document(source, &[(H1, 3, 3), (H1, 5, 5)]);
        assert!(reported(&doc).is_empty());

        // A heading of another level.
        let source = "## Sub\n\n# One\n\n# Two\n";
        let doc = document(source, &[(H2, 1, 1), (H1, 3, 3), (H1, 5, 5)]);
        assert!(reported(&doc).is_empty());
    }

    #[test]
    fn a_nested_first_level_one_heading_is_not_a_title() {
        let source = "> # Quoted\n\n# Two\n";
        let doc = document(
            source,
            &[(BlockKind::BlockQuote, 1, 1), (H1, 1, 1), (H1, 3, 3)],
        );
        assert!(reported(&doc).is_empty());

        // A container that ends before the heading does not hold it.
        let source = "- item\n\n# One\n\n# Two\n";
        let doc = document(source, &[(BlockKind::List, 1, 1), (H1, 3, 3), (H1, 5, 5)]);
        assert!(reported(&doc).is_empty(), "the list line is not blank");
    }

    #[test]
    fn an_html_block_with_text_is_not_a_comment() {
        let source = "<!-- a --> text\n\n# T\n\n# U\n";
        let doc = document(source, &[(BlockKind::Html, 1, 1), (H1, 3, 3), (H1, 5, 5)]);
        assert!(reported(&doc).is_empty());
    }

    #[test]
    fn a_document_without_level_one_headings_is_clean() {
        let source = "## One\n\n## Two\n";
        let doc = document(source, &[(H2, 1, 1), (H2, 3, 3)]);
        assert!(reported(&doc).is_empty());
        assert!(reported(&Document::from_source("text\n")).is_empty());
    }

    #[test]
    fn a_lone_title_is_clean() {
        let doc = document("# Title\n\ntext\n", &[(H1, 1, 1)]);
        assert!(reported(&doc).is_empty());
    }
}
