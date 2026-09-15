//! MD010 (`no-hard-tabs`): every run of consecutive tab characters is one finding at the column of
//! its first tab, on every line, including code blocks and code spans.

mod support;

#[test]
fn reports_one_finding_per_run_of_tabs() {
    let dir = support::temp_dir("md010-runs");
    let file = support::write_file(&dir, "tabs.md", "\tindented\na\t\tb\tc\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:1:1 MD010 hard tab\n{path}:2:2 MD010 hard tab\n{path}:2:5 MD010 hard tab\n"
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn space_indentation_is_clean() {
    let dir = support::temp_dir("md010-spaces");
    let file = support::write_file(
        &dir,
        "spaces.md",
        "- item\n  continued with spaces\n\n    indented code with spaces\n",
    );

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn reports_tabs_in_fenced_indented_code_and_code_spans() {
    let dir = support::temp_dir("md010-code");
    let source = [
        "Intro paragraph.",
        "",
        "```text",
        "\tfenced",
        "```",
        "",
        "\tindented code",
        "",
        "Inline `a\tb` span.",
    ]
    .join("\n")
        + "\n";
    let file = support::write_file(&dir, "code.md", &source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:4:1 MD010 hard tab\n{path}:7:1 MD010 hard tab\n{path}:9:10 MD010 hard tab\n"
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn tab_column_counts_characters() {
    let dir = support::temp_dir("md010-characters");
    let source = format!(
        "{}\n{}\n{}\n{}\n",
        "é".repeat(40) + " " + &"é".repeat(39),
        "é".repeat(81) + " é",
        "é\tx",
        "\t".to_string() + &"a".repeat(79),
    );
    let file = support::write_file(&dir, "characters.md", &source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:2:81 MD013 line too long (expected at most 80 characters, found 83)\n\
         {path}:3:2 MD010 hard tab\n\
         {path}:4:1 MD010 hard tab\n"
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}
