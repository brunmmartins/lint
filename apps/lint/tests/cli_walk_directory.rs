//! AC4: a directory with nested Markdown files, some clean and some dirty, is walked recursively;
//! every problem is reported and the exit code reflects whether any finding occurred.

mod support;

#[test]
fn nested_directory_reports_every_problem_and_exits_one() {
    let dir = support::temp_dir("walk-directory");
    support::write_file(&dir, "clean.md", "clean\n");
    let dirty = support::write_file(&dir, "nested/dirty.md", "trail  \n");
    support::write_file(&dir, "nested/deeper/also_clean.md", "clean\n");

    let output = support::run_lint(&[dir.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(1));
    let expected = format!("{}:1:6 MD009 trailing whitespace\n", dirty.display());
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_directory_with_only_clean_nested_files_exits_zero() {
    let dir = support::temp_dir("walk-directory-clean");
    support::write_file(&dir, "clean.md", "clean\n");
    support::write_file(&dir, "nested/also_clean.md", "clean\n");

    let output = support::run_lint(&[dir.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::stdout(&output), "");

    std::fs::remove_dir_all(&dir).unwrap();
}
