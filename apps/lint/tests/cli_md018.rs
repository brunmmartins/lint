//! MD018 (`no-missing-space-atx`): a line opening with `#` characters that no space follows is
//! reported, outside code and HTML blocks.

mod support;

const MESSAGE: &str = "MD018 no space after hash on atx style heading";

/// The stdout lines that name MD018, with the run's exit code.
fn md018_lines(output: &std::process::Output) -> (Option<i32>, Vec<String>) {
    let lines = support::stdout(output)
        .lines()
        .filter(|line| line.contains(" MD018 "))
        .map(ToString::to_string)
        .collect();
    (output.status.code(), lines)
}

#[test]
fn missing_space_after_hash() {
    let dir = support::temp_dir("md018-missing");
    let file = support::write_file(
        &dir,
        "hashes.md",
        "#Heading\n\n##Two\n\n#######seven\n\ntext\n#tag\n",
    );

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let path = file.display();
    let expected = format!(
        "{path}:1:1 {MESSAGE}\n{path}:3:1 {MESSAGE}\n{path}:5:1 {MESSAGE}\n{path}:8:1 {MESSAGE}\n"
    );
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn exempt_lines_are_not_reported() {
    let dir = support::temp_dir("md018-exempt");
    // One line per case, separated by blank lines, so line 19 is the only reported one.
    let cases = [
        "## Heading",
        "#",
        "#\tTab",
        "#Closed#",
        "#Closed ##  ",
        "  #indented",
        "#\u{FE0F}\u{20E3} keycap",
        "> #quote",
        "- #item",
        "#!shebang-like",
    ];
    let source = cases.join("\n\n") + "\n";
    let file = support::write_file(&dir, "exempt.md", &source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    let path = file.display();
    assert_eq!(
        md018_lines(&output),
        (Some(1), vec![format!("{path}:19:1 {MESSAGE}")])
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn code_and_html_blocks_are_ignored() {
    let dir = support::temp_dir("md018-blocks");
    let source = "```\n#fenced\n```\n\n    #indented\n\n<div>\n#html\n</div>\n\n~~~\n#open fence\n";
    let file = support::write_file(&dir, "blocks.md", source);

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}
