use std::num::NonZeroUsize;
use std::ops::Range;

use crate::{Block, BlockKind, LineSpan};

/// A parsed Markdown document, as the domain rules see it: 1-indexed lines with their line
/// terminators stripped, the count of consecutive trailing newline characters at end-of-file, and
/// the blocks whose line extents the rules need.
///
/// Line `n` is the text after the `(n - 1)`th `\n` of the source. The empty text after a final
/// `\n` is not a line.
///
/// Constructed only through [`Document::from_source`] and [`Document::from_source_and_blocks`],
/// which are infallible: any `&str` is a valid source to line-split, and any byte range maps to a
/// valid span of whole lines, so there is no invariant to reject at construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    lines: Vec<String>,
    trailing_newline_count: usize,
    source_is_empty: bool,
    blocks: Vec<Block>,
}

impl Document {
    /// Splits `source` into 1-indexed lines (the line terminator stripped from each), and records
    /// how many newline characters end the source consecutively: `0` means no trailing newline at
    /// all, `1` means exactly one, `2` or more means more than one.
    ///
    /// A `\r\n` line terminator has its `\r` stripped along with the `\n`, so line text never
    /// carries a trailing carriage return. The document has no blocks.
    #[must_use]
    pub fn from_source(source: &str) -> Self {
        if source.is_empty() {
            return Self {
                lines: Vec::new(),
                trailing_newline_count: 0,
                source_is_empty: true,
                blocks: Vec::new(),
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
            blocks: Vec::new(),
        }
    }

    /// Builds the document as [`from_source`](Self::from_source) does, and adds one [`Block`] for
    /// each `(kind, byte range)` in `blocks`, where the range is a byte range into `source`.
    ///
    /// Each range becomes the span of whole lines it touches, numbered as [`lines`](Self::lines)
    /// numbers them. The mapping accepts any range:
    ///
    /// - `start` and `end` are clamped to `source.len()`, and `end` is raised to at least `start`.
    /// - The first line holds the byte at `start`, and the last line holds the byte at `end - 1`.
    ///   An empty range spans only its first line.
    /// - Both lines are clamped to the last line, so a range that starts after the final newline
    ///   belongs to the last line. When the source has no lines, every block is dropped.
    ///
    /// [`blocks`](Self::blocks) keeps the blocks in order of first line; blocks with the same first
    /// line keep the order they were supplied in.
    ///
    /// Runs in `O(source + blocks × log lines)` time. An index of newline positions is built only
    /// when at least one block is supplied, and is dropped before this returns.
    #[must_use]
    pub fn from_source_and_blocks(
        source: &str,
        blocks: impl IntoIterator<Item = (BlockKind, Range<usize>)>,
    ) -> Self {
        let blocks = map_blocks(source, blocks);
        let mut document = Self::from_source(source);
        document.blocks = blocks;
        document
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

    /// The document's blocks, in non-decreasing order of first line. Blocks may nest or overlap.
    #[must_use]
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
}

/// Maps byte ranges to line spans. Kept apart from line splitting so the newline index is freed
/// before the line strings are allocated.
fn map_blocks(
    source: &str,
    blocks: impl IntoIterator<Item = (BlockKind, Range<usize>)>,
) -> Vec<Block> {
    let mut blocks = blocks.into_iter().peekable();
    if blocks.peek().is_none() {
        return Vec::new();
    }

    let newlines: Vec<usize> = source
        .bytes()
        .enumerate()
        .filter_map(|(offset, byte)| (byte == b'\n').then_some(offset))
        .collect();

    // The same count `from_source` produces: every newline ends a line, and text after the last
    // newline is one more line.
    let line_count = if source.is_empty() || source.ends_with('\n') {
        newlines.len()
    } else {
        newlines.len().saturating_add(1)
    };
    let Some(last_line) = NonZeroUsize::new(line_count) else {
        return Vec::new();
    };

    // The line holding the byte at `offset` is one more than the newlines strictly before it.
    let line_of = |offset: usize| {
        let newlines_before = newlines.partition_point(|&newline| newline < offset);
        NonZeroUsize::MIN
            .saturating_add(newlines_before)
            .min(last_line)
    };

    let mut mapped: Vec<Block> = blocks
        .map(|(kind, range)| {
            let start = range.start.min(source.len());
            let end = range.end.min(source.len()).max(start);
            let first = line_of(start);
            // `end > start` makes `end - 1` safe and keeps it at or after `start`.
            let last = if end > start { line_of(end - 1) } else { first };
            Block::new(kind, LineSpan::new(first, last))
        })
        .collect();
    mapped.sort_by_key(|block| block.lines().first());
    mapped
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

    /// `(kind, first line, last line)` for every block, in `blocks()` order.
    fn spans(doc: &Document) -> Vec<(BlockKind, usize, usize)> {
        doc.blocks()
            .iter()
            .map(|block| {
                let lines = block.lines();
                (block.kind(), lines.first().get(), lines.last().get())
            })
            .collect()
    }

    #[test]
    fn from_source_has_no_blocks() {
        assert!(
            Document::from_source("```\ncode\n```\n")
                .blocks()
                .is_empty()
        );
    }

    #[test]
    fn maps_byte_ranges_to_line_spans() {
        // Offsets: "para\n" 0..5, "\n" 5..6, "```\n" 6..10, "code\n" 10..15, "```\n" 15..19,
        // "\n" 19..20, "after\n" 20..26.
        let source = "para\n\n```\ncode\n```\n\nafter\n";
        let cases = [
            (6..18, (3, 5)),                       // closing fence without its newline
            (6..19, (3, 5)),                       // closing fence with its newline
            (8..12, (3, 4)),                       // starts and ends mid-line
            (10..10, (4, 4)),                      // empty range
            (0..26, (1, 7)),                       // whole source
            (Range { start: 15, end: 6 }, (5, 5)), // end before start
            (26..40, (7, 7)),                      // after the final newline
            (100..200, (7, 7)),                    // beyond the source
        ];

        for (range, (first, last)) in cases {
            let doc =
                Document::from_source_and_blocks(source, [(BlockKind::FencedCode, range.clone())]);
            assert_eq!(
                spans(&doc),
                [(BlockKind::FencedCode, first, last)],
                "range {range:?}"
            );
            assert_eq!(doc.lines(), Document::from_source(source).lines());
        }
    }

    #[test]
    fn crlf_ranges_use_the_same_line_numbers() {
        let source = "a\r\n```\r\n\r\n```\r\nb\r\n";
        let doc = Document::from_source_and_blocks(source, [(BlockKind::FencedCode, 3..15)]);
        assert_eq!(spans(&doc), [(BlockKind::FencedCode, 2, 4)]);
    }

    #[test]
    fn blocks_are_ordered_by_first_line_keeping_supplied_order_for_ties() {
        let source = "para\n\n```\ncode\n```\n\nafter\n";
        let doc = Document::from_source_and_blocks(
            source,
            [
                (BlockKind::IndentedCode, 10..15),
                (BlockKind::FencedCode, 6..19),
                (BlockKind::IndentedCode, 6..10),
            ],
        );
        assert_eq!(
            spans(&doc),
            [
                (BlockKind::FencedCode, 3, 5),
                (BlockKind::IndentedCode, 3, 3),
                (BlockKind::IndentedCode, 4, 4),
            ]
        );
    }

    #[test]
    fn source_without_lines_drops_every_block() {
        let doc = Document::from_source_and_blocks("", [(BlockKind::FencedCode, 0..10)]);
        assert!(doc.blocks().is_empty());
        assert!(doc.source_is_empty());
    }

    #[test]
    fn mapping_is_total_for_every_range() {
        let sources = ["", "\n", "a", "a\r\n\r\n", "```\n\n\n", "x\n\ny"];

        for source in sources {
            let expected = Document::from_source(source);
            let line_count = expected.lines().len();
            for start in 0..=source.len() + 2 {
                for end in 0..=source.len() + 2 {
                    let doc = Document::from_source_and_blocks(
                        source,
                        [(BlockKind::IndentedCode, start..end)],
                    );
                    assert_eq!(doc.lines(), expected.lines());
                    assert_eq!(
                        doc.trailing_newline_count(),
                        expected.trailing_newline_count()
                    );

                    if line_count == 0 {
                        assert!(doc.blocks().is_empty(), "{source:?} {start}..{end}");
                        continue;
                    }
                    let [(_, first, last)] = spans(&doc)[..] else {
                        panic!("expected one block for {source:?} {start}..{end}");
                    };
                    assert!(
                        1 <= first && first <= last && last <= line_count,
                        "{source:?} {start}..{end} gave {first}..={last}"
                    );

                    // The first line holds the byte at `start`, counted naively.
                    let clamped = start.min(source.len());
                    let naive_first = 1 + source.as_bytes()[..clamped]
                        .iter()
                        .filter(|&&b| b == b'\n')
                        .count();
                    assert_eq!(
                        first,
                        naive_first.min(line_count),
                        "{source:?} {start}..{end}"
                    );
                }
            }
        }
    }
}
