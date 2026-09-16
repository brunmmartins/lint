//! MD025 (`single-h1`): a second top-level heading is reported only when the document's first
//! content is a top-level heading.

mod support;

const MESSAGE: &str = "MD025 multiple top-level headings in the same document";

#[test]
fn extra_top_level_headings() {
    let dir = support::temp_dir("md025-extra");
    let source = "# Title\n\n## Part\n\n# Second\n\nOther\n=====\n\n- # In list\n";
    let file = support::write_file(&dir, "extra.md", source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    assert_eq!(
        support::stdout(&output),
        format!("{path}:5:1 {MESSAGE}\n{path}:7:1 {MESSAGE}\n{path}:10:1 {MESSAGE}\n")
    );
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn title_after_a_comment_block() {
    let dir = support::temp_dir("md025-comment");
    let file = support::write_file(&dir, "comment.md", "<!-- c -->\n\n# Title\n\n# Two\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        support::stdout(&output),
        format!("{}:5:1 {MESSAGE}\n", file.display())
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn no_title_cases() {
    let cases = [
        ("paragraph-first", "Intro.\n\n# One\n\n# Two\n"),
        ("other-level-first", "## Sub\n\n# One\n\n# Two\n"),
        ("nested-first", "> # Quoted\n\n# Two\n"),
        ("html-with-text-first", "<!-- a --> text\n\n# T\n\n# U\n"),
        (
            "link-definition-first",
            "[a]: https://example.com/\n\n# One\n\n# Two\n",
        ),
        ("quoted-comment-first", "> <!-- c -->\n\n# One\n\n# Two\n"),
    ];

    for (name, source) in cases {
        let dir = support::temp_dir(&format!("md025-{name}"));
        let file = support::write_file(&dir, "case.md", source);

        let output = support::run_lint(&[file.to_str().unwrap()]);

        assert_eq!(output.status.code(), Some(0), "case {name}");
        assert_eq!(support::stdout(&output), "", "case {name}");
        assert_eq!(support::stderr(&output), "", "case {name}");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
