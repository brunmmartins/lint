//! The size bound on one file sits exactly at 10 MiB: a file of that size is linted as usual, and a
//! file one byte larger is rejected with exit 2 and nothing on stdout.

mod support;

const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;

#[test]
fn a_file_of_exactly_the_bound_is_linted() {
    let dir = support::temp_dir("size-bound-exact");
    // One unbroken line: long, but without whitespace to wrap at, so it has no finding.
    let contents = format!("{}\n", "a".repeat(MAX_FILE_BYTES - 1));
    let path = support::write_file(&dir, "bound.md", &contents);

    let output = support::run_lint(&[path.to_str().unwrap()]);

    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(support::stderr(&output), "");
    assert_eq!(support::stdout(&output), "");
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn a_file_one_byte_over_the_bound_is_rejected() {
    let dir = support::temp_dir("size-bound-over");
    let contents = format!("{}\n", "a".repeat(MAX_FILE_BYTES));
    let path = support::write_file(&dir, "over.md", &contents);

    let output = support::run_lint(&[path.to_str().unwrap()]);

    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(
        support::stderr(&output),
        format!(
            "{}: exceeds the maximum size of 10485760 bytes\n",
            path.display()
        )
    );
    assert_eq!(support::stdout(&output), "");
    assert_eq!(output.status.code(), Some(2));
}
