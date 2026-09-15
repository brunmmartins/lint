use lint_application::MarkdownParser;

/// Drives `pulldown-cmark`'s parser to completion over the source, for its structural
/// robustness/validity property (its block parsing is non-recursive, so hostile input such as
/// deeply nested block structure cannot overflow the stack here). The event stream itself is not
/// yet retained as domain structure; [`lint_domain::Document`] is built from the source's own
/// lines.
#[derive(Debug, Default, Clone, Copy)]
pub struct PulldownMarkdownParser;

impl MarkdownParser for PulldownMarkdownParser {
    fn parse(&self, source: &str) -> lint_domain::Document {
        for _event in pulldown_cmark::Parser::new(source) {
            // Drained only to exercise the parser's own robustness guarantee; the current rules
            // (MD009, MD047) operate on raw line text, not on the event stream.
        }
        lint_domain::Document::from_source(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
