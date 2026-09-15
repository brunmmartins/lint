//! A Markdown file that follows every rule's default produces no output and exits 0.

mod support;

#[test]
fn conforming_document_is_clean() {
    let dir = support::temp_dir("conforming-document");
    let long_url_line = "Read the full reference at https://example.com/".to_string()
        + &"long-path-segment/".repeat(4)
        + "index.html";
    assert!(long_url_line.chars().count() > 80);
    let source = [
        "# Conforming document",
        "",
        "This paragraph is wrapped at eighty characters or fewer, so that no line in it",
        "is too long for the default limit.",
        "",
        "```text",
        "first",
        "",
        "",
        "second",
        "```",
        "",
        "| Name | Value |",
        "| --- | --- |",
        "| a | 1 |",
        "",
        "- first item",
        "- second item",
        "",
        long_url_line.as_str(),
    ]
    .join("\n")
        + "\n";
    let file = support::write_file(&dir, "conforming.md", &source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");
    assert_eq!(output.status.code(), Some(0));

    std::fs::remove_dir_all(&dir).unwrap();
}
