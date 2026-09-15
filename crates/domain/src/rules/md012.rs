use crate::{Document, Finding, FindingMessage, Location, Rule, RuleId};

/// MD012 (`no-multiple-blanks`): more than one consecutive blank line outside code blocks.
///
/// A line is blank when it is empty or holds only whitespace. A line inside a code block, or a
/// line that is not blank, ends the run. Each blank line whose run count exceeds the maximum is
/// reported at column 1, with the count so far.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md012;

/// The most consecutive blank lines allowed.
const MAXIMUM: usize = 1;

impl Rule for Md012 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        let mut findings = Vec::new();
        // `blocks()` is ordered by first line, so one pass over lines and blocks together finds
        // the lines covered by code, however the blocks nest or overlap.
        let mut pending_blocks = doc.blocks().iter().peekable();
        let mut code_through_line = 0;
        let mut blank_run = 0_usize;

        for (index, line) in doc.lines().iter().enumerate() {
            let line_number = index + 1;
            while let Some(block) =
                pending_blocks.next_if(|block| block.lines().first().get() <= line_number)
            {
                if block.kind().is_code() {
                    code_through_line = code_through_line.max(block.lines().last().get());
                }
            }

            let is_code = line_number <= code_through_line;
            if is_code || !line.trim().is_empty() {
                blank_run = 0;
                continue;
            }

            blank_run = blank_run.saturating_add(1);
            if blank_run <= MAXIMUM {
                continue;
            }
            // `line_number` is a 1-based count over an existing line and the column is 1, so
            // `Location::new` cannot fail; a failure would be a counting bug here.
            if let Ok(location) = Location::new(line_number, 1) {
                findings.push(Finding::new(
                    RuleId::Md012,
                    location,
                    FindingMessage::MultipleBlankLines {
                        maximum: MAXIMUM,
                        found: blank_run,
                    },
                ));
            }
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;
    use std::ops::Range;

    /// `(line, found)` for every finding.
    fn reported(doc: &Document) -> Vec<(usize, usize)> {
        Md012
            .check(doc)
            .iter()
            .map(|finding| {
                assert_eq!(finding.rule(), RuleId::Md012);
                assert_eq!(finding.location().col().get(), 1);
                let FindingMessage::MultipleBlankLines { maximum, found } = finding.message()
                else {
                    panic!("unexpected message {:?}", finding.message());
                };
                assert_eq!(maximum, 1);
                (finding.location().line().get(), found)
            })
            .collect()
    }

    /// The byte range from the start of line `first` to the end of line `last` (1-based), without
    /// the final newline, in a source whose every line ends with `\n`.
    fn line_range(source: &str, first: usize, last: usize) -> Range<usize> {
        let mut starts = vec![0];
        starts.extend(
            source
                .bytes()
                .enumerate()
                .filter(|&(_, byte)| byte == b'\n')
                .map(|(offset, _)| offset + 1),
        );
        starts[first - 1]..starts[last] - 1
    }

    #[test]
    fn reports_each_blank_beyond_maximum() {
        let doc = Document::from_source("one\n\n\n\ntwo\n");
        assert_eq!(reported(&doc), [(3, 2), (4, 3)]);
    }

    #[test]
    fn whitespace_only_line_is_blank() {
        let doc = Document::from_source("one\n\n  \ntwo\n");
        assert_eq!(reported(&doc), [(3, 2)]);
        let doc = Document::from_source("one\n\t\n\u{3000}\ntwo\n");
        assert_eq!(reported(&doc), [(3, 2)]);
    }

    #[test]
    fn single_blank_lines_are_clean() {
        let doc = Document::from_source("one\n\ntwo\n\nthree\n");
        assert!(reported(&doc).is_empty());
    }

    #[test]
    fn text_after_final_newline_is_not_a_line() {
        assert!(reported(&Document::from_source("abc\n\n")).is_empty());
        assert_eq!(reported(&Document::from_source("abc\n\n\n")), [(3, 2)]);
    }

    #[test]
    fn code_block_lines_reset_the_count() {
        // Lines: 1 text, 2 blank, 3 fence, 4 blank, 5 blank, 6 fence, 7 blank, 8 blank, 9 text.
        let source = "text\n\n```\n\n\n```\n\n\nafter\n";
        let doc = Document::from_source_and_blocks(
            source,
            [(BlockKind::FencedCode, line_range(source, 3, 6))],
        );
        assert_eq!(reported(&doc), [(8, 2)]);

        // A blank line right before a code block does not carry its count into the block.
        let source = "text\n\n    code\n\n\n    more\n";
        let doc = Document::from_source_and_blocks(
            source,
            [(BlockKind::IndentedCode, line_range(source, 3, 6))],
        );
        assert!(reported(&doc).is_empty());
    }

    #[test]
    fn nested_and_overlapping_spans_match_a_naive_scan() {
        // Mostly blank lines, so that code coverage decides nearly every finding.
        let source = "\n\n\nx\n\n\n\n\n";
        let line_count = 8;
        let mut spans = Vec::new();
        for first in 1..=line_count {
            for last in first..=line_count {
                spans.push((first, last));
            }
        }

        let mut checked = 0;
        for &a in &spans {
            for &b in &spans {
                for &c in [None, Some((2, 3)), Some((5, 8))].iter() {
                    // Supply the spans out of order to exercise the document's ordering.
                    let mut supplied = vec![b, a];
                    supplied.extend(c);
                    let doc = Document::from_source_and_blocks(
                        source,
                        supplied.iter().map(|&(first, last)| {
                            (BlockKind::FencedCode, line_range(source, first, last))
                        }),
                    );

                    let mut expected = Vec::new();
                    let mut run = 0;
                    for (index, line) in doc.lines().iter().enumerate() {
                        let number = index + 1;
                        let in_code = doc.blocks().iter().any(|block| {
                            block.kind().is_code()
                                && block.lines().first().get() <= number
                                && number <= block.lines().last().get()
                        });
                        if in_code || !line.trim().is_empty() {
                            run = 0;
                        } else {
                            run += 1;
                            if run > 1 {
                                expected.push((number, run));
                            }
                        }
                    }

                    assert_eq!(reported(&doc), expected, "spans {supplied:?}");
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, spans.len() * spans.len() * 3);
    }
}
