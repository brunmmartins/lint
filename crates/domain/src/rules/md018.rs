use crate::{Document, Finding, FindingMessage, Location, Rule, RuleId};

/// MD018 (`no-missing-space-atx`): a line opens with `#` characters that no space follows, so it
/// was probably meant as a heading but does not render as one.
///
/// This reads lines rather than parsed structure, so a run of seven or more `#` characters and a
/// `#` word that continues a paragraph are both reported even though neither is a heading. Lines
/// inside fenced code blocks, indented code blocks, and HTML blocks are never reported.
///
/// A line is left alone when its `#` characters are followed by a space or a tab, when it holds
/// only `#` characters, when its last non-blank character is `#`, when it does not open with `#`
/// at column 1, and when it opens with the keycap emoji. The finding is reported at column 1.
#[derive(Debug, Default, Clone, Copy)]
pub struct Md018;

/// The keycap emoji, which opens with a `#` but is not a heading marker.
const KEYCAP: &str = "#\u{FE0F}\u{20E3}";

impl Rule for Md018 {
    fn check(&self, doc: &Document) -> Vec<Finding> {
        let mut findings = Vec::new();
        // `blocks()` is ordered by first line, so one pass over lines and blocks together finds
        // the lines covered by code and HTML, however the blocks nest or overlap.
        let mut pending_blocks = doc.blocks().iter().peekable();
        let mut excluded_through = 0;

        for (index, line) in doc.lines().iter().enumerate() {
            let line_number = index + 1;
            while let Some(block) =
                pending_blocks.next_if(|block| block.lines().first().get() <= line_number)
            {
                let kind = block.kind();
                if kind.is_code() || kind.is_html() {
                    excluded_through = excluded_through.max(block.lines().last().get());
                }
            }

            if line_number <= excluded_through || !misses_space_after_hash(line) {
                continue;
            }
            // `line_number` is a 1-based count over an existing line and the column is 1, so
            // `Location::new` cannot fail; a failure would be a counting bug here.
            if let Ok(location) = Location::new(line_number, 1) {
                findings.push(Finding::new(
                    RuleId::Md018,
                    location,
                    FindingMessage::MissingSpaceAfterHash,
                ));
            }
        }
        findings
    }
}

/// Whether `line` opens with `#` characters that no space follows.
fn misses_space_after_hash(line: &str) -> bool {
    let after_hashes = line.trim_start_matches('#');
    if after_hashes.len() == line.len() {
        return false;
    }
    match after_hashes.chars().next() {
        None | Some(' ' | '\t') => return false,
        Some(_) => {}
    }
    if line.trim_end().ends_with('#') {
        return false;
    }
    !line.starts_with(KEYCAP)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlockKind, HeadingLevel};

    /// The line numbers reported for `source`, which has no blocks.
    fn reported(source: &str) -> Vec<usize> {
        reported_in(&Document::from_source(source))
    }

    fn reported_in(doc: &Document) -> Vec<usize> {
        Md018
            .check(doc)
            .iter()
            .map(|finding| {
                assert_eq!(finding.rule(), RuleId::Md018);
                assert_eq!(finding.location().col().get(), 1);
                assert_eq!(finding.message(), FindingMessage::MissingSpaceAfterHash);
                finding.location().line().get()
            })
            .collect()
    }

    /// The byte range covering lines `first` through `last` (1-based) of a source whose every line
    /// ends with `\n`.
    fn line_range(source: &str, first: usize, last: usize) -> std::ops::Range<usize> {
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
    fn reports_hash_followed_by_text() {
        assert_eq!(reported("#Heading\n\n##Two\n"), [1, 3]);
    }

    #[test]
    fn a_run_of_seven_hashes_and_a_continuation_line_are_reported() {
        assert_eq!(reported("#######seven\n\ntext\n#tag\n"), [1, 4]);
    }

    #[test]
    fn exempt_lines_are_not_reported() {
        let exempt = [
            "## Heading",
            "#",
            "###",
            "#\tTab",
            "# x",
            "#Closed#",
            "#Closed ##  ",
            "#x#\u{3000}",
            "  #indented",
            "> #quote",
            "- #item",
            "#\u{FE0F}\u{20E3} keycap",
            "text",
            "",
        ];
        for line in exempt {
            let source = format!("{line}\n");
            assert!(reported(&source).is_empty(), "{line:?}");
        }

        assert_eq!(reported("#!shebang-like\n"), [1]);
    }

    #[test]
    fn code_and_html_spans_hide_lines() {
        let source = "#a\n#b\n#c\n#d\n";
        let hiding = [
            BlockKind::FencedCode,
            BlockKind::IndentedCode,
            BlockKind::Html,
        ];
        for kind in hiding {
            let doc = Document::from_source_and_blocks(source, [(kind, line_range(source, 2, 3))]);
            assert_eq!(reported_in(&doc), [1, 4], "{kind:?}");
        }
    }

    #[test]
    fn headings_and_containers_do_not_hide_lines() {
        let source = "#a\n#b\n#c\n#d\n";
        let showing = [
            BlockKind::Heading(HeadingLevel::H1),
            BlockKind::BlockQuote,
            BlockKind::List,
        ];
        for kind in showing {
            let doc = Document::from_source_and_blocks(source, [(kind, line_range(source, 2, 3))]);
            assert_eq!(reported_in(&doc), [1, 2, 3, 4], "{kind:?}");
        }
    }

    #[test]
    fn nested_and_overlapping_spans_match_a_naive_scan() {
        let source = "#a\n#b\n#c\n#d\n#e\n";
        let line_count = 5;
        let mut spans = Vec::new();
        for first in 1..=line_count {
            for last in first..=line_count {
                spans.push((first, last));
            }
        }

        let mut checked = 0;
        for &a in &spans {
            for &b in &spans {
                // Supply the spans out of order to exercise the document's ordering, and mix in a
                // kind that hides nothing.
                let supplied = [
                    (BlockKind::Html, b),
                    (BlockKind::BlockQuote, a),
                    (BlockKind::FencedCode, a),
                ];
                let doc = Document::from_source_and_blocks(
                    source,
                    supplied
                        .iter()
                        .map(|&(kind, (first, last))| (kind, line_range(source, first, last))),
                );

                let expected: Vec<usize> = (1..=line_count)
                    .filter(|&number| {
                        !doc.blocks().iter().any(|block| {
                            (block.kind().is_code() || block.kind().is_html())
                                && block.lines().first().get() <= number
                                && number <= block.lines().last().get()
                        })
                    })
                    .collect();

                assert_eq!(reported_in(&doc), expected, "spans {supplied:?}");
                checked += 1;
            }
        }
        assert_eq!(checked, spans.len() * spans.len());
    }
}
