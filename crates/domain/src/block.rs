use std::num::NonZeroUsize;

/// A 1-based, inclusive range of [`Document`](crate::Document) lines.
///
/// `first()` is never greater than `last()`. Spans are built only by
/// [`Document::from_source_and_blocks`](crate::Document::from_source_and_blocks), which also
/// guarantees that `last()` is never beyond the document's last line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineSpan {
    first: NonZeroUsize,
    last: NonZeroUsize,
}

impl LineSpan {
    /// Builds a span, raising `last` to `first` if it is lower, so the ordering invariant holds for
    /// any pair of line numbers.
    pub(crate) fn new(first: NonZeroUsize, last: NonZeroUsize) -> Self {
        Self {
            first,
            last: last.max(first),
        }
    }

    /// The first line in the span.
    #[must_use]
    pub fn first(&self) -> NonZeroUsize {
        self.first
    }

    /// The last line in the span, which is never before [`first`](Self::first).
    #[must_use]
    pub fn last(&self) -> NonZeroUsize {
        self.last
    }
}

/// One of the six heading levels Markdown defines, from `H1` (a document title) to `H6`.
///
/// The set is closed: Markdown has exactly these six levels, and a seventh run of `#` characters
/// is ordinary text rather than a heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HeadingLevel {
    /// A level-1 heading, written `# text` or underlined with `=`.
    H1,
    /// A level-2 heading, written `## text` or underlined with `-`.
    H2,
    /// A level-3 heading, written `### text`.
    H3,
    /// A level-4 heading, written `#### text`.
    H4,
    /// A level-5 heading, written `##### text`.
    H5,
    /// A level-6 heading, written `###### text`.
    H6,
}

impl HeadingLevel {
    /// The level as the number `1` to `6` that messages print.
    #[must_use]
    pub fn number(self) -> u8 {
        match self {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            HeadingLevel::H3 => 3,
            HeadingLevel::H4 => 4,
            HeadingLevel::H5 => 5,
            HeadingLevel::H6 => 6,
        }
    }

    /// The next level down, or `None` for the deepest level, which has none.
    #[must_use]
    pub fn deeper(self) -> Option<Self> {
        match self {
            HeadingLevel::H1 => Some(HeadingLevel::H2),
            HeadingLevel::H2 => Some(HeadingLevel::H3),
            HeadingLevel::H3 => Some(HeadingLevel::H4),
            HeadingLevel::H4 => Some(HeadingLevel::H5),
            HeadingLevel::H5 => Some(HeadingLevel::H6),
            HeadingLevel::H6 => None,
        }
    }
}

/// What kind of Markdown block a [`Block`] is.
///
/// New kinds may be added. Code that asks a question about a block, such as whether its lines are
/// code, uses a query method like [`is_code`](Self::is_code) rather than matching on variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BlockKind {
    /// A code block opened by a backtick or tilde fence. It spans the opening fence line through
    /// the closing fence line, or to the end of the document when the fence is never closed.
    FencedCode,
    /// A code block made of indented lines, including any blank lines between its chunks.
    IndentedCode,
    /// A heading, written with leading `#` characters or underlined with `=` or `-`. It spans
    /// every line of the heading, an underline included.
    Heading(HeadingLevel),
    /// A block of raw HTML, such as a `<div>` element or a comment, spanning every line of it.
    Html,
    /// A block quote, spanning its lines. Blocks inside it are reported separately.
    BlockQuote,
    /// An ordered or unordered list, spanning all of its items. Items get no block of their own.
    List,
}

impl BlockKind {
    /// Whether every line of a block of this kind is code.
    #[must_use]
    pub fn is_code(self) -> bool {
        match self {
            BlockKind::FencedCode | BlockKind::IndentedCode => true,
            BlockKind::Heading(_) | BlockKind::Html | BlockKind::BlockQuote | BlockKind::List => {
                false
            }
        }
    }

    /// The heading level of a heading block, or `None` for any other kind.
    #[must_use]
    pub fn heading_level(self) -> Option<HeadingLevel> {
        match self {
            BlockKind::Heading(level) => Some(level),
            BlockKind::FencedCode
            | BlockKind::IndentedCode
            | BlockKind::Html
            | BlockKind::BlockQuote
            | BlockKind::List => None,
        }
    }

    /// Whether every line of a block of this kind is raw HTML.
    #[must_use]
    pub fn is_html(self) -> bool {
        match self {
            BlockKind::Html => true,
            BlockKind::FencedCode
            | BlockKind::IndentedCode
            | BlockKind::Heading(_)
            | BlockKind::BlockQuote
            | BlockKind::List => false,
        }
    }

    /// Whether a block of this kind holds other blocks, so a block inside its line span is nested
    /// rather than at the top level of the document.
    #[must_use]
    pub fn is_container(self) -> bool {
        match self {
            BlockKind::BlockQuote | BlockKind::List => true,
            BlockKind::FencedCode
            | BlockKind::IndentedCode
            | BlockKind::Heading(_)
            | BlockKind::Html => false,
        }
    }
}

/// One Markdown block of a [`Document`](crate::Document): its kind and the whole lines it spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    kind: BlockKind,
    lines: LineSpan,
}

impl Block {
    pub(crate) fn new(kind: BlockKind, lines: LineSpan) -> Self {
        Self { kind, lines }
    }

    /// The kind of block.
    #[must_use]
    pub fn kind(&self) -> BlockKind {
        self.kind
    }

    /// The lines the block spans.
    #[must_use]
    pub fn lines(&self) -> LineSpan {
        self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(number: usize) -> NonZeroUsize {
        NonZeroUsize::new(number).unwrap()
    }

    #[test]
    fn span_keeps_first_and_last() {
        let span = LineSpan::new(line(2), line(5));
        assert_eq!(span.first(), line(2));
        assert_eq!(span.last(), line(5));
    }

    #[test]
    fn span_never_ends_before_it_starts() {
        let span = LineSpan::new(line(4), line(1));
        assert_eq!(span.first(), line(4));
        assert_eq!(span.last(), line(4));
    }

    #[test]
    fn every_code_kind_is_code() {
        assert!(BlockKind::FencedCode.is_code());
        assert!(BlockKind::IndentedCode.is_code());
    }

    const LEVELS: [HeadingLevel; 6] = [
        HeadingLevel::H1,
        HeadingLevel::H2,
        HeadingLevel::H3,
        HeadingLevel::H4,
        HeadingLevel::H5,
        HeadingLevel::H6,
    ];

    #[test]
    fn heading_levels_number_and_deepen() {
        let numbers: Vec<u8> = LEVELS.iter().map(|level| level.number()).collect();
        assert_eq!(numbers, [1, 2, 3, 4, 5, 6]);

        let deeper: Vec<Option<HeadingLevel>> = LEVELS.iter().map(|level| level.deeper()).collect();
        assert_eq!(
            deeper,
            [
                Some(HeadingLevel::H2),
                Some(HeadingLevel::H3),
                Some(HeadingLevel::H4),
                Some(HeadingLevel::H5),
                Some(HeadingLevel::H6),
                None,
            ]
        );

        let mut ordered = LEVELS;
        ordered.reverse();
        ordered.sort();
        assert_eq!(ordered, LEVELS);
    }

    #[test]
    fn each_kind_answers_its_queries() {
        // (kind, is_code, heading level, is_html, is_container)
        let cases = [
            (BlockKind::FencedCode, true, None, false, false),
            (BlockKind::IndentedCode, true, None, false, false),
            (
                BlockKind::Heading(HeadingLevel::H3),
                false,
                Some(HeadingLevel::H3),
                false,
                false,
            ),
            (BlockKind::Html, false, None, true, false),
            (BlockKind::BlockQuote, false, None, false, true),
            (BlockKind::List, false, None, false, true),
        ];

        for (kind, is_code, heading_level, is_html, is_container) in cases {
            assert_eq!(kind.is_code(), is_code, "{kind:?}");
            assert_eq!(kind.heading_level(), heading_level, "{kind:?}");
            assert_eq!(kind.is_html(), is_html, "{kind:?}");
            assert_eq!(kind.is_container(), is_container, "{kind:?}");
        }

        for level in LEVELS {
            assert_eq!(
                BlockKind::Heading(level).heading_level(),
                Some(level),
                "{level:?}"
            );
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn block_stays_fixed_size() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<Block>();
        assert!(size_of::<Block>() <= 24, "{} bytes", size_of::<Block>());
    }

    #[test]
    fn block_accessors_return_constructed_values() {
        let span = LineSpan::new(line(1), line(3));
        let block = Block::new(BlockKind::IndentedCode, span);
        assert_eq!(block.kind(), BlockKind::IndentedCode);
        assert_eq!(block.lines(), span);
    }
}
