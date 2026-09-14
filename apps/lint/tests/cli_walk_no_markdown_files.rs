//! AC9: a directory containing no Markdown files prints nothing, prints no error, and exits 0.

mod support;

#[test]
fn directory_with_no_markdown_files_is_silent_and_exits_zero() {
    let dir = support::temp_dir("walk-no-markdown-files");
    support::write_file(&dir, "readme.txt", "not markdown\n");
    support::write_file(&dir, "notes.rs", "fn main() {}\n");

    let output = support::run_lint(&[dir.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}
