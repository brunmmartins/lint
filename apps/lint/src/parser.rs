use std::ops::Range;

use lint_application::MarkdownParser;
use lint_domain::{BlockKind, Document};
use pulldown_cmark::{CodeBlockKind, Event, Tag};

/// Parses Markdown with `pulldown-cmark` and records where each fenced and indented code block
/// starts and ends, so rules can tell code lines from other lines.
///
/// The parser runs with CommonMark defaults and no extensions. Its block parsing is
/// non-recursive, so hostile input such as deeply nested block structure cannot overflow the stack
/// here. Only block kinds and byte ranges leave this module; the [`Document`] turns them into line
/// spans.
#[derive(Debug, Default, Clone, Copy)]
pub struct PulldownMarkdownParser;

impl MarkdownParser for PulldownMarkdownParser {
    fn parse(&self, source: &str) -> Document {
        // Collect the ranges before building the document, so the parser's own structures are
        // freed before the document's lines are allocated.
        let blocks: Vec<(BlockKind, Range<usize>)> = pulldown_cmark::Parser::new(source)
            .into_offset_iter()
            .filter_map(|(event, range)| match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                    Some((BlockKind::FencedCode, range))
                }
                Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                    Some((BlockKind::IndentedCode, range))
                }
                _ => None,
            })
            .collect();
        Document::from_source_and_blocks(source, blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `(kind, first line, last line)` for every block the parser reports.
    fn spans(source: &str) -> Vec<(BlockKind, usize, usize)> {
        PulldownMarkdownParser
            .parse(source)
            .blocks()
            .iter()
            .map(|block| {
                let lines = block.lines();
                (block.kind(), lines.first().get(), lines.last().get())
            })
            .collect()
    }

    #[test]
    fn parsing_hostile_markdown_does_not_panic() {
        let deeply_nested = "> ".repeat(10_000) + "text";
        let long_line = "a".repeat(1_000_000);
        let inputs = [
            deeply_nested.as_str(),
            long_line.as_str(),
            "",
            "# heading\n\ntext\n",
        ];

        for input in inputs {
            // The absence of a panic is the property under test.
            let _document = PulldownMarkdownParser.parse(input);
        }
    }

    #[test]
    fn parse_builds_a_document_from_the_source_lines() {
        let document = PulldownMarkdownParser.parse("one\ntwo\n");
        assert_eq!(document.lines(), ["one", "two"]);
        assert_eq!(document.trailing_newline_count(), 1);
        assert!(document.blocks().is_empty());
    }

    #[test]
    fn maps_backtick_and_tilde_fences() {
        let source = "text\n\n```text\ncode\n```\n\n\n~~~\nmore\n\n~~~\nafter\n";
        assert_eq!(
            spans(source),
            [
                (BlockKind::FencedCode, 3, 5),
                (BlockKind::FencedCode, 8, 11)
            ]
        );
    }

    #[test]
    fn indented_block_spans_interior_blank_lines_only() {
        // Lines: 1 para, 2 blank, 3 code a, 4 blank, 5 blank, 6 code b, 7 blank, 8 blank, 9 after.
        let source = "para\n\n    code a\n\n\n    code b\n\n\nafter\n";
        assert_eq!(spans(source), [(BlockKind::IndentedCode, 3, 6)]);
    }

    #[test]
    fn unclosed_fence_spans_to_last_line() {
        assert_eq!(spans("```\n\n\n"), [(BlockKind::FencedCode, 1, 3)]);
        assert_eq!(
            spans("text\n\n```\ncode\n\n\nend"),
            [(BlockKind::FencedCode, 3, 7)]
        );
    }

    #[test]
    fn fence_in_list_item() {
        let source = "- item\n\n  ```\n  code\n  ```\n\nafter\n";
        assert_eq!(spans(source), [(BlockKind::FencedCode, 3, 5)]);
    }

    #[test]
    fn crlf_source() {
        let source = "text\r\n\r\n```\r\n\r\n\r\n```\r\n\r\n\r\nafter\r\n";
        assert_eq!(spans(source), [(BlockKind::FencedCode, 3, 6)]);
    }

    #[test]
    fn html_blocks_are_not_code() {
        assert!(spans("<div>\n\n</div>\n").is_empty());
    }
}
