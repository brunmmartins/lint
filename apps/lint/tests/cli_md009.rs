//! AC2: a file with a trailing-space line reports one MD009 finding and exits 1.

mod support;

#[test]
fn trailing_spaces_report_md009_and_exit_one() {
    let dir = support::temp_dir("md009");
    let file = support::write_file(&dir, "dirty.md", "trail  \n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!("{}:1:6 MD009 trailing whitespace\n", file.display());
    assert_eq!(support::stdout(&output), expected);
    assert_eq!(support::stderr(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn leading_and_internal_spaces_alone_report_nothing() {
    let dir = support::temp_dir("md009-negative");
    let file = support::write_file(&dir, "clean.md", "  leading and internal only\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn trailing_tab_reports_md009() {
    let dir = support::temp_dir("md009-tab");
    let file = support::write_file(&dir, "dirty.md", "trail\t\n");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!("{}:1:6 MD009 trailing whitespace\n", file.display());
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}
