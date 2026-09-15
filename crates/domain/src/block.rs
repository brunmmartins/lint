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
}

impl BlockKind {
    /// Whether every line of a block of this kind is code.
    #[must_use]
    pub fn is_code(self) -> bool {
        match self {
            BlockKind::FencedCode | BlockKind::IndentedCode => true,
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

    #[test]
    fn block_accessors_return_constructed_values() {
        let span = LineSpan::new(line(1), line(3));
        let block = Block::new(BlockKind::IndentedCode, span);
        assert_eq!(block.kind(), BlockKind::IndentedCode);
        assert_eq!(block.lines(), span);
    }
}
