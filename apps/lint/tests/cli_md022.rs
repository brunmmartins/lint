//! MD022 (`blanks-around-headings`): a heading with no blank line directly above or below it is
//! reported on its first line, the one above before the one below.

mod support;

const ABOVE: &str = "MD022 missing blank line above heading";
const BELOW: &str = "MD022 missing blank line below heading";

/// The stdout lines that name MD022, with the run's exit code.
fn md022_lines(output: &std::process::Output) -> (Option<i32>, Vec<String>) {
    let lines = support::stdout(output)
        .lines()
        .filter(|line| line.contains(" MD022 "))
        .map(ToString::to_string)
        .collect();
    (output.status.code(), lines)
}

#[test]
fn missing_blank_lines_around_a_heading() {
    let dir = support::temp_dir("md022-around");
    let file = support::write_file(&dir, "around.md", "Intro\n# Heading\nText\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    assert_eq!(
        support::stdout(&output),
        format!("{path}:2:1 {ABOVE}\n{path}:2:1 {BELOW}\n")
    );
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn headings_at_the_file_edges() {
    let dir = support::temp_dir("md022-edges");
    let file = support::write_file(&dir, "edges.md", "# Top\ntext\n\ntext\n## Two\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    assert_eq!(
        support::stdout(&output),
        format!("{path}:1:1 {BELOW}\n{path}:5:1 {ABOVE}\n")
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn heading_on_a_last_line_without_newline() {
    let dir = support::temp_dir("md022-last-line");
    let file = support::write_file(&dir, "last.md", "text\n\n# Last");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(md022_lines(&output), (Some(1), Vec::new()));

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn setext_headings() {
    let dir = support::temp_dir("md022-setext");
    let clean = support::write_file(&dir, "clean.md", "# First\n\nSetext\n------\n\n## Last\n");
    let output = support::run_lint(&[clean.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");

    let under = support::write_file(&dir, "under.md", "para\n\nTitle\n===\nbody\n");
    let output = support::run_lint(&[under.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        support::stdout(&output),
        format!("{}:3:1 {BELOW}\n", under.display())
    );

    let long = support::write_file(&dir, "long.md", "para\n\nline a\nline b\n===\nnext\n");
    let output = support::run_lint(&[long.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        support::stdout(&output),
        format!("{}:3:1 {BELOW}\n", long.display())
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn block_quotes_comments_and_lists() {
    let dir = support::temp_dir("md022-quotes");
    let source = "> text\n> # Quoted\n>\n> more\n\n<!-- note -->\n# After comment\n\n- item\n- ## In list\n\n  body\n";
    let file = support::write_file(&dir, "quotes.md", source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    assert_eq!(
        support::stdout(&output),
        format!("{path}:2:1 {ABOVE}\n{path}:10:1 {ABOVE}\n")
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn text_before_an_unclosed_comment_is_not_blank() {
    let dir = support::temp_dir("md022-unclosed");
    let source = "# A\n  \n## B\n>\n### C\n<!-- x -->\n#### D\nx <!-- y\n##### E\n";
    let file = support::write_file(&dir, "unclosed.md", source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    let path = file.display();
    assert_eq!(
        md022_lines(&output),
        (
            Some(1),
            vec![format!("{path}:7:1 {BELOW}"), format!("{path}:9:1 {ABOVE}"),]
        )
    );

    std::fs::remove_dir_all(&dir).unwrap();
}
