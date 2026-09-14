//! AC7: a nonexistent path prints an error naming the path to stderr and exits 2, without
//! crashing and without silently skipping the path (and without skipping a sibling valid path).

mod support;

#[test]
fn nonexistent_path_prints_error_to_stderr_and_exits_two() {
    let dir = support::temp_dir("nonexistent-path");
    let missing = dir.join("does-not-exist.md");

    let output = support::run_lint(&[missing.to_str().unwrap()]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(support::stdout(&output), "");
    let stderr = support::stderr(&output);
    assert!(stderr.contains(&missing.display().to_string()));

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_nonexistent_path_does_not_stop_a_sibling_valid_path_from_being_linted() {
    let dir = support::temp_dir("nonexistent-path-sibling");
    let missing = dir.join("does-not-exist.md");
    let dirty = support::write_file(&dir, "dirty.md", "trail  \n");

    let output = support::run_lint(&[missing.to_str().unwrap(), dirty.to_str().unwrap()]);

    // A fault takes exit-code precedence over a finding (ADR-0006), but the valid path's finding
    // is still printed.
    assert_eq!(output.status.code(), Some(2));
    let expected = format!("{}:1:6 MD009 trailing whitespace\n", dirty.display());
    assert_eq!(support::stdout(&output), expected);

    std::fs::remove_dir_all(&dir).unwrap();
}
