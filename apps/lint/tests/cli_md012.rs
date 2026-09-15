//! MD012 (`no-multiple-blanks`): each blank line beyond the first in a run of consecutive blank
//! lines is reported, except inside fenced and indented code blocks.

mod support;

fn blank_lines_message(found: usize) -> String {
    format!("MD012 multiple consecutive blank lines (expected at most 1, found {found})")
}

#[test]
fn reports_excess_blank_lines() {
    let dir = support::temp_dir("md012-excess");
    let file = support::write_file(&dir, "blanks.md", "one\n\n\n\ntwo\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:3:1 {}\n{path}:4:1 {}\n",
        blank_lines_message(2),
        blank_lines_message(3)
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn whitespace_only_line_counts_as_blank() {
    let dir = support::temp_dir("md012-whitespace");
    let file = support::write_file(&dir, "spaces.md", "one\n\n  \ntwo\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:3:1 MD009 trailing whitespace\n{path}:3:1 {}\n",
        blank_lines_message(2)
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn single_blank_lines_are_clean() {
    let dir = support::temp_dir("md012-single");
    let file = support::write_file(&dir, "single.md", "one\n\ntwo\n\nthree\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn blank_lines_inside_code_blocks_are_ignored() {
    let cases = [
        (
            "backtick-fence",
            "Intro.\n\n```\ncode\n\n\nmore code\n```\n",
        ),
        ("tilde-fence", "Intro.\n\n~~~\ncode\n\n\nmore code\n~~~\n"),
        ("indented-code", "Intro.\n\n    code a\n\n\n    code b\n"),
        (
            "unclosed-fence",
            "Intro.\n\n```\ncode\n\n\nstill code at end of file\n",
        ),
    ];

    for (name, source) in cases {
        let dir = support::temp_dir(&format!("md012-{name}"));
        let file = support::write_file(&dir, "code.md", source);

        let output = support::run_lint(&[file.to_str().unwrap()]);

        assert_eq!(output.status.code(), Some(0), "case {name}");
        assert_eq!(support::stdout(&output), "", "case {name}");
        assert_eq!(support::stderr(&output), "", "case {name}");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}

#[test]
fn blank_lines_after_closing_fence_are_reported() {
    let dir = support::temp_dir("md012-after-fence");
    let file = support::write_file(&dir, "after.md", "```\ncode\n```\n\n\nafter\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!("{}:5:1 {}\n", file.display(), blank_lines_message(2));
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn end_of_file_blank_lines_with_md047() {
    let dir = support::temp_dir("md012-end-of-file");
    let two = support::write_file(&dir, "two.md", "abc\n\n");
    let three = support::write_file(&dir, "three.md", "abc\n\n\n");

    let output = support::run_lint(&[two.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let expected = format!(
        "{}:2:1 MD047 more than one trailing newline\n",
        two.display()
    );
    assert_eq!(support::stdout(&output), expected);

    let output = support::run_lint(&[three.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let path = three.display();
    let expected = format!(
        "{path}:3:1 {}\n{path}:3:1 MD047 more than one trailing newline\n",
        blank_lines_message(2)
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}
