use std::ops::Range;

use lint_application::MarkdownParser;
use lint_domain::{BlockKind, Document, HeadingLevel};
use pulldown_cmark::{CodeBlockKind, Event, Tag};

/// Parses Markdown with `pulldown-cmark` and records where each code block, heading, HTML block,
/// block quote, and list starts and ends, so rules can tell those lines apart.
///
/// The parser runs with CommonMark defaults and no extensions. Its block parsing is
/// non-recursive, so hostile input such as deeply nested block structure cannot overflow the stack
/// here. Only block kinds and byte ranges leave this module; the [`Document`] turns them into line
/// spans, and decides what is nested from the spans themselves.
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
                Event::Start(Tag::Heading { level, .. }) => {
                    Some((BlockKind::Heading(heading_level(level)), range))
                }
                Event::Start(Tag::HtmlBlock) => Some((BlockKind::Html, range)),
                Event::Start(Tag::BlockQuote(_)) => Some((BlockKind::BlockQuote, range)),
                Event::Start(Tag::List(_)) => Some((BlockKind::List, range)),
                _ => None,
            })
            .collect();
        Document::from_source_and_blocks(source, blocks)
    }
}

/// Translates the parser's heading level into the domain's, so no parser type leaves this module.
fn heading_level(level: pulldown_cmark::HeadingLevel) -> HeadingLevel {
    match level {
        pulldown_cmark::HeadingLevel::H1 => HeadingLevel::H1,
        pulldown_cmark::HeadingLevel::H2 => HeadingLevel::H2,
        pulldown_cmark::HeadingLevel::H3 => HeadingLevel::H3,
        pulldown_cmark::HeadingLevel::H4 => HeadingLevel::H4,
        pulldown_cmark::HeadingLevel::H5 => HeadingLevel::H5,
        pulldown_cmark::HeadingLevel::H6 => HeadingLevel::H6,
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
        assert_eq!(
            spans(source),
            [(BlockKind::List, 1, 6), (BlockKind::FencedCode, 3, 5)]
        );
    }

    #[test]
    fn crlf_source() {
        let source = "text\r\n\r\n```\r\n\r\n\r\n```\r\n\r\n\r\nafter\r\n";
        assert_eq!(spans(source), [(BlockKind::FencedCode, 3, 6)]);
    }

    #[test]
    fn html_blocks_are_not_code() {
        let blocks = spans("<div>\n\n</div>\n");
        assert_eq!(blocks, [(BlockKind::Html, 1, 1), (BlockKind::Html, 3, 3)]);
        assert!(blocks.iter().all(|&(kind, _, _)| !kind.is_code()));
    }

    #[test]
    fn maps_atx_and_setext_heading_levels_and_spans() {
        let source = "# one\n\n## two\n\n### three\n\n#### four\n\n##### five\n\n###### six\n";
        assert_eq!(
            spans(source),
            [
                (BlockKind::Heading(HeadingLevel::H1), 1, 1),
                (BlockKind::Heading(HeadingLevel::H2), 3, 3),
                (BlockKind::Heading(HeadingLevel::H3), 5, 5),
                (BlockKind::Heading(HeadingLevel::H4), 7, 7),
                (BlockKind::Heading(HeadingLevel::H5), 9, 9),
                (BlockKind::Heading(HeadingLevel::H6), 11, 11),
            ]
        );

        // Seven hashes are a paragraph, and four spaces of indent are code.
        assert_eq!(
            spans("####### seven\n\n    # indented\n"),
            [(BlockKind::IndentedCode, 3, 3)]
        );
    }

    #[test]
    fn multi_line_setext_heading_spans_its_underline() {
        assert_eq!(
            spans("Title\n=====\n\nSub\n---\n"),
            [
                (BlockKind::Heading(HeadingLevel::H1), 1, 2),
                (BlockKind::Heading(HeadingLevel::H2), 4, 5),
            ]
        );
        assert_eq!(
            spans("line a\nline b\n===\n"),
            [(BlockKind::Heading(HeadingLevel::H1), 1, 3)]
        );
    }

    #[test]
    fn headings_nested_in_lists_and_block_quotes_are_mapped() {
        let source = "# A\n\n- item\n\n  ### Nested\n\n> #### Quoted\n";
        let blocks = spans(source);
        assert!(
            blocks.contains(&(BlockKind::Heading(HeadingLevel::H3), 5, 5)),
            "{blocks:?}"
        );
        assert!(
            blocks.contains(&(BlockKind::Heading(HeadingLevel::H4), 7, 7)),
            "{blocks:?}"
        );
    }

    #[test]
    fn maps_html_blocks() {
        assert_eq!(
            spans("<div>\nstill html\n</div>\n\nafter\n"),
            [(BlockKind::Html, 1, 3)]
        );
        // Two comment lines in a row are two blocks.
        assert_eq!(
            spans("<!-- a -->\n<!-- b -->\n"),
            [(BlockKind::Html, 1, 1), (BlockKind::Html, 2, 2)]
        );
    }

    #[test]
    fn an_unclosed_comment_html_block_runs_to_the_last_line() {
        assert_eq!(
            spans("<!-- open\nstill\nstill\n"),
            [(BlockKind::Html, 1, 3)]
        );
    }

    #[test]
    fn maps_block_quotes_and_lists() {
        assert_eq!(spans("> quoted\n"), [(BlockKind::BlockQuote, 1, 1)]);
        assert_eq!(spans("- a\n- b\n"), [(BlockKind::List, 1, 2)]);
        assert_eq!(spans("10. a\n"), [(BlockKind::List, 1, 1)]);
        assert_eq!(
            spans("> a\n> > b\n"),
            [(BlockKind::BlockQuote, 1, 2), (BlockKind::BlockQuote, 2, 2)]
        );
    }

    #[test]
    fn container_spans_end_before_a_following_top_level_heading() {
        let shapes = [
            "- a\n# H\n",
            "- a\n\n# H\n",
            "- a\nlazy\n# H\n",
            "> a\n  # H\n",
            "10. a\n   # H\n",
            "- a\r\n\r\n# H\r\n",
        ];
        for source in shapes {
            let blocks = spans(source);
            let heading = blocks
                .iter()
                .find(|&&(kind, _, _)| kind.heading_level().is_some())
                .copied();
            let Some((_, heading_first, _)) = heading else {
                panic!("no heading in {source:?}: {blocks:?}");
            };
            for &(kind, _, last) in &blocks {
                assert!(
                    !kind.is_container() || last < heading_first,
                    "container reaches the heading in {source:?}: {blocks:?}"
                );
            }
        }
    }
}
