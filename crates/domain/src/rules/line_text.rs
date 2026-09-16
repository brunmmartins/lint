//! Text tests the heading rules share: whether a line counts as blank around a heading, and
//! whether an HTML block holds nothing but one HTML comment.
//!
//! Both are pure functions over line text. They scan each line once, so a very long line costs
//! time proportional to its length and nothing more.

/// Whether `line` counts as blank for the blank-line test around a heading.
///
/// A line is blank when it is empty or holds only whitespace, and also when nothing is left of it
/// once HTML comment text and block-quote markers are taken out. So `>`, `<!-- note -->`, and
/// `> <!-- note -->` are all blank, while `text <!-- note -->` is not.
///
/// An unclosed `<!--` hides the rest of the line, and an unmatched `-->` hides everything before
/// it, matching how the reference implementation strips comments.
pub(crate) fn is_blank_line(line: &str) -> bool {
    if line.trim().is_empty() {
        return true;
    }

    let mut rest = line;
    let mut in_comment = false;
    let mut content = false;

    while !rest.is_empty() {
        if in_comment {
            if let Some(after) = rest.strip_prefix("-->") {
                in_comment = false;
                rest = after;
                continue;
            }
        } else if let Some(after) = rest.strip_prefix("<!--") {
            in_comment = true;
            rest = after;
            continue;
        } else if let Some(after) = rest.strip_prefix("-->") {
            // A close with no open before it hides everything up to here.
            content = false;
            rest = after;
            continue;
        }

        let mut characters = rest.chars();
        let Some(character) = characters.next() else {
            break;
        };
        if !in_comment && character != '>' && !character.is_whitespace() {
            content = true;
        }
        rest = characters.as_str();
    }

    !content
}

/// Whether the HTML block spanning `lines` holds one HTML comment and nothing else, so that it may
/// still precede a document's title.
///
/// The block's text is its lines joined with newlines and trimmed. It must open with `<!--`, close
/// with `-->`, and hold a body that neither opens with `>` or `->` nor ends with `-`, which is how
/// the reference implementation recognises a comment.
pub(crate) fn is_comment_block(lines: &[String]) -> bool {
    match lines {
        [] => false,
        // Joining is skipped for the common one-line comment, so a very long single line is never
        // copied. A joined block costs at most the bytes of that block, once, and is freed here.
        [single] => is_comment_text(single.trim()),
        _ => is_comment_text(lines.join("\n").trim()),
    }
}

/// Whether `text`, already trimmed, is a single HTML comment.
fn is_comment_text(text: &str) -> bool {
    if !(text.starts_with("<!--") && text.ends_with("-->")) {
        return false;
    }
    // Shorter than both markers together means they overlap, as in `<!-->`, leaving no body.
    let body = text.get(4..text.len().saturating_sub(3)).unwrap_or("");
    !(body.starts_with('>') || body.starts_with("->") || body.ends_with('-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_line_cases() {
        let blank = [
            "",
            "   ",
            "\t",
            ">",
            "> >",
            "  >  ",
            "<!-- note -->",
            "> <!-- note -->",
            "<!-- a --> <!-- b -->",
            "<!---->",
        ];
        for line in blank {
            assert!(is_blank_line(line), "{line:?} should be blank");
        }

        let not_blank = ["text", "> text", "x <!-- y -->", "<!-- y --> x", "-", "<"];
        for line in not_blank {
            assert!(!is_blank_line(line), "{line:?} should not be blank");
        }
    }

    #[test]
    fn an_unmatched_close_hides_earlier_text() {
        assert!(is_blank_line("a-->"));
        assert!(is_blank_line("text --> "));
        assert!(!is_blank_line("a--> b"));
    }

    #[test]
    fn an_unclosed_open_hides_the_rest() {
        assert!(is_blank_line("<!--a"));
        assert!(is_blank_line("<!-- a <!-- b"));
        assert!(!is_blank_line("x <!-- y"));
    }

    #[test]
    fn a_million_character_comment_line_is_blank() {
        // The scan visits each byte once, so this finishes in the test's own time budget.
        let line = "<!-- -->".repeat(125_000);
        assert_eq!(line.len(), 1_000_000);
        assert!(is_blank_line(&line));
    }

    /// Builds the line vector an HTML block spanning `lines` would have.
    fn block(lines: &[&str]) -> Vec<String> {
        lines.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn comment_block_cases() {
        let comments = [
            vec!["<!-- c -->"],
            vec!["   <!-- c -->  "],
            vec!["<!---->"],
            vec!["<!-->"],
            vec!["<!--", "c", "-->"],
            vec!["<!--", "", "-->"],
        ];
        for lines in comments {
            assert!(is_comment_block(&block(&lines)), "{lines:?}");
        }

        let not_comments: Vec<Vec<&str>> = vec![
            vec![],
            vec!["<div>"],
            vec!["<!-- c --> text"],
            vec!["text <!-- c -->"],
            vec!["<!-- c -->", "<div>"],
            vec!["<!-->-->"],
            vec!["<!--->-->"],
            vec!["<!----->"],
        ];
        for lines in not_comments {
            assert!(!is_comment_block(&block(&lines)), "{lines:?}");
        }
    }

    #[test]
    fn a_multi_line_comment_block_joins_its_lines() {
        // Joined, the text opens and closes correctly; line by line, neither line does.
        assert!(is_comment_block(&block(&["<!-- start", "end -->"])));
        assert!(!is_comment_block(&block(&["<!-- start", "end --> x"])));
    }
}
