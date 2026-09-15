//! A file missing (or with more than) a single trailing newline reports one MD047 finding
//! and exits 1.

mod support;

#[test]
fn missing_trailing_newline_reports_md047_and_exits_one() {
    let dir = support::temp_dir("md047-missing");
    let file = support::write_file(&dir, "dirty.md", "no newline at end");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!(
        "{}:1:18 MD047 missing single trailing newline\n",
        file.display()
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn multiple_trailing_newlines_report_md047_and_exit_one() {
    let dir = support::temp_dir("md047-multiple");
    let file = support::write_file(&dir, "dirty.md", "content\n\n\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    // Line 3 is also a second consecutive blank line, which MD012 reports at the same location,
    // before MD047.
    let expected = format!(
        "{}:3:1 MD012 multiple consecutive blank lines (expected at most 1, found 2)\n\
         {}:3:1 MD047 more than one trailing newline\n",
        file.display(),
        file.display()
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn exactly_one_trailing_newline_reports_nothing() {
    let dir = support::temp_dir("md047-clean");
    let file = support::write_file(&dir, "clean.md", "content\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn empty_file_reports_nothing() {
    let dir = support::temp_dir("md047-empty");
    let file = support::write_file(&dir, "empty.md", "");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}
