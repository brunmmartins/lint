//! A single clean Markdown file prints nothing and exits 0.

mod support;

#[test]
fn clean_single_file_prints_nothing_and_exits_zero() {
    let dir = support::temp_dir("walk-single-file");
    let file = support::write_file(&dir, "clean.md", "clean content\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}
