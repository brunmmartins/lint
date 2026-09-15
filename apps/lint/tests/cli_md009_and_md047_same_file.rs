//! A file with both a trailing-space line and a missing final newline reports both MD009
//! and MD047 findings, with independent, correct locations, and exits 1.

mod support;

#[test]
fn both_rules_report_independently_for_one_file() {
    let dir = support::temp_dir("md009-and-md047");
    // Line 1 has trailing spaces, and the whole file is missing its final newline.
    let file = support::write_file(&dir, "both.md", "trail  \nclean");

    let output = support::run_lint(&[file.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!(
        "{}:1:6 MD009 trailing whitespace\n{}:2:6 MD047 missing single trailing newline\n",
        file.display(),
        file.display()
    );
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}
